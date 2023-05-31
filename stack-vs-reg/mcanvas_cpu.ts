import {Layout, LayoutProps} from "@motion-canvas/2d/lib/components"
import {Rect, Vector2} from "@motion-canvas/core/lib/types";
import {initial, property} from "@motion-canvas/2d/lib/decorators"
import {all, delay, sequence, waitFor} from "@motion-canvas/core/lib/flow";
import {Signal} from "@motion-canvas/core/lib/utils";
import {drawRoundRect} from "@motion-canvas/2d/lib/utils";
import {linear} from "@motion-canvas/core/lib/tweening";
import {decorate, threadable} from "@motion-canvas/core/lib/decorators"
import {ThreadGenerator} from "@motion-canvas/core/lib/threading";


export class Cpu extends Layout {
    public code_view:  CodeView;
    public controller: Controller;
    public decoder:    Decoder;
    public math:       MathUnit;
    public registers:  Registers;
    public memory:     MemoryUnit;

    public program: Instruction[]
    public decode_counter:  number;
    public blocked: boolean;

    public decode_limit: number;
    public anim_decode_duration: number;
    public anim_read_duration: number;
    public anim_exec_duration: number;

    public constructor(props?: LayoutProps) {
        super(props);

        this.setWidth(1700);
        this.setHeight(800);

        let x0 = -850;
        let y0 = -400;

        this.code_view = new CodeView({
            x: x0, y: y0,
            width: 350, height: 475,
        });

        this.decoder = new Decoder({
            x: x0 + 400, y: y0,
            width: 300, height: 320,
        });
        this.registers = new Registers({
            x: x0 + 400, y: y0 + 370,
            width: 300, height: 430,
        })
        this.controller = new Controller({
            x: x0 + 750, y: y0,
            width: 450, height: 800,
        });
        this.math = new MathUnit({
            x: x0 + 1250, y: y0,
            width: 450, height: 320,
        });
        this.memory = new MemoryUnit({
            x: x0 + 1250, y: y0 + 370,
            width: 450, height: 430,
        });
        this.add([this.code_view, this.controller, this.decoder, this.math, this.registers, this.memory]);

        this.program = [];
        this.decode_counter = 0;
        this.blocked = false;

        this.decode_limit = 4;
        this.anim_decode_duration = 0.6;
        this.anim_read_duration = 0.4;
        this.anim_exec_duration = 0.75;
    }


    public* decoded_to_controller() {
        if(this.blocked) {
            return;
        }

        let ctrl = this.controller;
        let slots_remaining = ctrl.slots.length - ctrl.in_flight;

        let instructions: Instruction[] = [];
        for(const s of this.decoder.slots) {
            let i = s.instruction;
            if(i && slots_remaining) {
                instructions.push(i);
                s.instruction = null;
                slots_remaining -= 1;

                if(i.jump_kind) {
                    break;
                }
            }
        }

        yield* this.controller.add_instructions(instructions);
    }

    public* dispatch() {
        let regs: (Register | Instruction)[] = this.registers.registers.slice();

        let anims: ThreadGenerator[] = [];
        
        this.controller.slots.forEach(s => {
            if(!s.instruction) { return }

            let i = s.instruction;

            let blocked = false;
            let values = i.reads.map(r => regs[r]);
            values.forEach(v => {
                if(v instanceof Instruction && !v.done) {
                    blocked = true;
                }
            });
            i.blocked = blocked;
            i.read_values = values;
            i.invalidate();

            if(i.writes !== null) {
                console.assert(!i.done || !!i.wb, i);
                regs[i.writes] = i;
            }

            if(!blocked && !i.done) {
                if(i.unit == "math") {
                    let slot = this.math.slots.find(s => !s.instruction);
                    if(slot) {
                        let o = new ProgressOverlay(slot);
                        o.max_opacity(0.9);
                        slot.instruction = i;
                        anims.push(this.dispatch_instruction(s, slot));
                    }
                }
                else if(i.unit == "memory") {
                    console.assert(!"unimplemented");
                }
                else if(i.unit == "none") {
                    let o = new ProgressOverlay(s);
                    o.max_opacity(0.9);
                }
                else {
                    console.assert(false);
                }
            }
        });

        yield* all(waitFor(0.2), ...anims);
    }

    public* execute() {
        let overlays: ProgressOverlay[] = [];
        let instructions: Instruction[] = [];

        this.controller.slots.forEach(s => {
            let i = s.instruction;
            if(i && i.unit == "none" && !i.done && !i.blocked) {
                overlays.push(s.overlay);
                i.run();
                instructions.push(i);
            }
        });

        this.math.slots.forEach(s => {
            let i = s.instruction;
            if(i && !i.done && !i.blocked) {
                overlays.push(s.overlay);
                i.run();
                instructions.push(i);
            }
        });

        yield* all(
            ...overlays.map(o => o?.run(this.anim_exec_duration)),
            ...instructions.map(i => i.anim_reads(this.anim_read_duration)),
        );
    }

    public* gather() {
        yield* all(...this.math.slots.map(s => {
            let i = s.instruction;
            if(i && i.done) {
                return this.gather_instruction(s);
            }
        }));
    }

    public* write_back() {
        let slots = this.controller.slots;

        let n = slots.findIndex(s => {
            let i = s.instruction;
            return !i || !i.done
        });

        yield* sequence(0.15,
            ...this.controller.slots.slice(0, n)
            .map(s => {
                let i = s.instruction;
                if(i.done && i.wb) {
                    return this.write_back_instruction(i);
                }
            }));
    }

    public* retire() {
        let slots = this.controller.slots;

        let n = slots.findIndex(s => {
            let i = s.instruction;
            return !i || !i.done
        });

        this.blocked = true;
        let end = this.controller.in_flight;
        if(end > 0) {
            let last = slots[end - 1].instruction;
            this.decode_counter = last.pc + 1;

            if(last.jump_kind) {
                let kind = last.jump_kind;
                if(kind == "jump") {
                    this.blocked = false;
                    this.decode_counter = last.jump_target;
                }
                else {
                    kind = kind.slice(1);
                    let negate = false;
                    if(kind.startsWith("n")) {
                        negate = true;
                        kind = kind.slice(1);
                    }

                    if(last.done) {
                        let v = last.read_values[0];
                        let value = v instanceof Register ? v.value : v.wb.value;
                        this.blocked = false;
                        if(value == kind && !negate || value != kind && negate) {
                            this.decode_counter = last.jump_target;
                        }
                    }
                }
            }
            else {
                this.blocked = false;
            }
        }

        yield* this.controller.retire_instructions(n);

        this.code_view.set_counters(
               this.controller.slots[0].instruction?.pc
            ?? this.decode_counter,
            this.decode_counter,
            this.decode_limit
        );
    }

    public* run(max_iters: number = 1000) {
        let iters = 0;
        while(iters < max_iters) {
            iters += 1;

            if(this.blocked
            && this.decode_counter >= this.program.length
            && this.controller.in_flight == 0
            ) { break }

            let new_instructions = this.program.slice(
                this.decode_counter,
                this.decode_counter + this.decode_limit,
            );

            yield* this.decoder.decode(new_instructions,
                this.decode_limit,
                this.anim_decode_duration
            );

            yield* this.decoded_to_controller();
            yield* waitFor(0.2);

            yield* this.dispatch();

            yield* this.execute();
            yield* waitFor(0.1);

            yield* this.gather();
            yield* waitFor(0.2);

            yield* this.write_back();
            yield* this.retire();
        }
    }


    public* dispatch_instruction(from: BasicSlot, to: BasicSlot) {
        to.tag = from.tag;
        to.instruction = from.instruction;

        yield* all(
            to.instruction.position(position_in_parent(to, -40), 0.4),
            delay(0.2, to.tag_opacity(1, 0.2)),
        );
    }


    public* gather_instruction(from: BasicSlot) {
        let to = this.controller.slots.find(s => s.tag == from.tag);
        from.instruction = null;

        yield* all(
            to.instruction.position(position_in_parent(to, -40), 0.4),
            from.tag_opacity(0, 0.2),
        );
        from.tag = "";
    }

    public* write_back_instruction(instruction: Instruction) {
        let wb = instruction.wb;
        console.assert(!!wb);
        instruction.wb = null;

        let pos = instruction.position().add(wb.position());
        wb.remove();
        this.add(wb);
        wb.position(pos);

        let register = this.registers.registers[wb.register];

        let target = position_in_parent(register);
        yield* all(
            wb.position(target, 0.4),
            delay(0.3, wb.opacity(0, 0.1)),
            delay(0.3, register.set_value(wb.value)),
        );
        wb.remove();
    }

    public parse(asm: string): Instruction[] {
        let pc = 0;
        let labels: Record<string, number> = {};

        let lines = asm.trim().split("\n").map(line => line.trim());

        lines.forEach(line => {
            if(line.length < 1) { return }

            if(line.endsWith(":")) {
                let label = line.split(":")[0];
                labels[label] = pc;
            }
            else {
                pc += 1;
            }
        });

        let result: Instruction[] = [];
        lines.forEach(line => {
            if(line.length < 1 || line.endsWith(":")) { return }

            let asm = line.split(" ").filter(s => s.length > 0).join(" ");

            let parts = line.replace(",", "").split(" ").filter(part => part.length > 0);

            let op = parts[0];
            if(op == "set") {
                let r = parseInt(parts[1].slice(1));
                let v = parseFloat(parts[2].slice(1));
                result.push(new Instruction({
                    asm, pc: result.length,
                    reads: [],
                    writes: r,
                    exec: exec_set(v),
                    unit: "none",
                }));
            }
            else if(op == "copy") {
                let dst = parseInt(parts[1].slice(1));
                let src = parseInt(parts[2].slice(1));
                result.push(new Instruction({
                    asm, pc: result.length,
                    reads: [src],
                    writes: dst,
                    exec: exec_copy,
                    unit: "none",
                }));
            }
            else if(op == "add") {
                let dst = parseInt(parts[1].slice(1));
                let src1 = parseInt(parts[2].slice(1));
                if(parts[3].startsWith("#")) {
                    let src2 = parseFloat(parts[3].slice(1));
                    result.push(new Instruction({
                        asm, pc: result.length,
                        reads: [src1],
                        writes: dst,
                        exec: exec_add_imm(src2),
                        unit: "math",
                    }));
                }
                else {
                    let src2 = parseInt(parts[3].slice(1));
                    result.push(new Instruction({
                        asm, pc: result.length,
                        reads: [src1, src2],
                        writes: dst,
                        exec: exec_add,
                        unit: "math",
                    }));
                }
            }
            else if(op == "cmp") {
                let src1 = parseInt(parts[1].slice(1));
                let src2 = parseInt(parts[2].slice(1));
                result.push(new Instruction({
                    asm, pc: result.length,
                    reads: [src1, src2],
                    writes: 11,
                    exec: exec_cmp,
                    unit: "math",
                }));
            }
            else if(op == "jump" || op == "jlt" || op == "jnlt") {
                let jump_target = labels[parts[1]];
                console.assert(typeof jump_target === "number");

                let reads = op == "jump" ? [] : [11];
                result.push(new Instruction({
                    asm, pc: result.length,
                    reads,
                    writes: null,
                    exec: exec_none,
                    unit: "none",
                    jump_kind: op, jump_target,
                }));
            }
            else {
                console.assert(!"invalid asm", line);
            }
        });
        return result;
    }

    public set_code(asm: string) {
        this.program = this.parse(asm);

        let lines = asm.split("\n");
        lines = lines.slice(lines.findIndex(line => line.length > 0));
        while(lines.length && lines[lines.length-1].trim().length < 1) {
            lines.pop();
        }

        this.code_view.set_code(lines);
        this.code_view.set_counters(0, 0, this.decode_limit);
        this.decode_counter = 0;
        this.blocked = false;
    }

    public set_regs(values: number[]) {
        let regs = this.registers.registers;
        values.slice(0, 10).forEach((v, i) => {
            regs[i].value = v.toString();
        });
    }
}
decorate(Cpu.prototype.decoded_to_controller, threadable());
decorate(Cpu.prototype.dispatch, threadable());
decorate(Cpu.prototype.execute, threadable());
decorate(Cpu.prototype.gather, threadable());
decorate(Cpu.prototype.retire, threadable());
decorate(Cpu.prototype.run, threadable());
decorate(Cpu.prototype.dispatch_instruction, threadable());
decorate(Cpu.prototype.gather_instruction, threadable());
decorate(Cpu.prototype.write_back_instruction, threadable());


let exec_set = (value: number) => (values: number[]) => value;

let exec_copy = (values: number[]) => values[0];

let exec_add = (values: number[]) => values[0] + values[1];

let exec_add_imm = (imm: number) => (values: number[]) => values[0] + imm;

let exec_cmp = (values: number[]) => {
    let [a, b] = values
    if(a == b) { return "eq" }
    if(a <  b) { return "lt" }
    if(a >  b) { return "gt" }
    return "ne";
};

let exec_none = (values: number[]) => null as number | null;



function position_in_parent(node: Layout, dx?: number, dy?: number): Vector2 {
    let parent = node.parent() as Layout;
    let x = parent.position.x();
    let y = parent.position.y();
    return new Vector2(
        x + node.position.x() + (dx ?? 0),
        y + node.position.y() + (dy ?? 0));
}

function draw_component(name: string, rect: Rect, ctx: CanvasRenderingContext2D) {
    ctx.beginPath();
    drawRoundRect(ctx, rect, 9);
    ctx.closePath();

    ctx.save();
    ctx.fillStyle = "#333949";
    ctx.shadowBlur = 7;
    ctx.shadowOffsetY = 3;
    ctx.shadowColor = "rgba(0, 0, 0, 0.5)";
    ctx.fill();
    ctx.restore();

    ctx.save();
    ctx.strokeStyle = "#394051";
    ctx.lineWidth = 2;
    ctx.stroke();
    ctx.restore();

    ctx.font = "bold 24px Source Code Pro"
    ctx.fillStyle = "#7D8391";
    ctx.fillText(name, rect.x + 20, rect.y + 36);
}


export interface InstructionProps extends LayoutProps {
    asm: string;
    reads: number[];
    writes: number | null;
    exec: (values: number[]) => number | string | null;
    unit: "math" | "memory" | "none";
    pc: number;
    jump_kind?: string;
    jump_target?: number;
}

export class Instruction extends Layout {
    public asm: string;
    public reads: number[];
    public read_values: (Register | Instruction)[];
    public writes: number | null;
    public exec: (values: number[]) => number | string | null;
    public unit: "math" | "memory" | "none";
    public pc: number;
    public jump_kind: string | null;
    public jump_target: number | null;
    public tag: string;
    public wb: WriteBack | null;
    public blocked: boolean;
    public done: boolean;
    @initial(0) @property()
    public declare state: Signal<number>;

    public constructor(props?: InstructionProps) {
        super(props);
        this.setWidth(225);
        this.setHeight(25);
        this.asm = props.asm;
        this.reads = props.reads;
        this.writes = props.writes;
        this.read_values = [];
        this.exec = props.exec;
        this.unit = props.unit;
        this.pc = props.pc;
        this.jump_kind = props.jump_kind ?? null;
        this.jump_target = props.jump_target ?? null;
        this.tag = "";
        this.wb = null;
        this.blocked = false;
        this.done = false;
    }

    protected override draw(ctx: CanvasRenderingContext2D): void {
        let _ = this.state();
        let x = -this.size().x/2;
        let fill = "#BFBDB6";
        if(this.done) { fill = "#858684" }
        if(this.blocked) { fill = "#D9461E" }
        ctx.fillStyle = fill;
        ctx.font = (this.done ? "italic " : "") + "24px Source Code Pro"
        ctx.textBaseline = "middle";
        ctx.fillText(this.asm, x, 2);
        super.draw(ctx);
    }

    public run(): WriteBack | null {
        console.assert(!this.blocked);
        console.assert(!this.done);

        let values = this.read_values.map(v => {
            let value = v instanceof Register ? v.value : v.wb.value;
            return parseFloat(value);
        });

        let result = this.exec(values);
        this.done = true;
        this.invalidate()

        if(this.writes === null) {
            return null;
        }

        // this is so cursed, lol
        let value = typeof result === "number"
            ? parseFloat(result.toFixed(2)).toString()
            : result;

        let wb = new WriteBack({ register: this.writes, value });
        wb.position.x(this.size.x()/2);
        wb.opacity(0.0);
        this.wb = wb;
        this.add(wb);
        return wb;
    }

    public* anim_reads(duration: number) {
        yield* all(...this.read_values.map((r, index) => {
            let pos, text;
            if(r instanceof Register) {
                pos  = position_in_parent(r);
                text = r.value;
            }
            else {
                pos  = position_in_parent(r.wb, r.wb.size().x/2);
                text = r.wb.value;
            }

            let read = new RegRead({
                x: pos.x,
                y: pos.y,
                text,
            });
            this.parent().add(read);

            let tx = this.position.x() + this.size.x()/2 + index*45 + 30;
            let ty = this.position.y();
            return read.move_and_die([tx, ty], duration);
        }));
    }

    public invalidate() {
        this.state(this.state() + 1);
    }
}
decorate(Instruction.prototype.anim_reads, threadable());


export interface WriteBackProps extends LayoutProps {
    register: number;
    value:    string;
}

export class WriteBack extends Layout {
    public text: string;
    public register: number;
    public value:    string;

    public constructor(props?: WriteBackProps) {
        super(props);
        this.offset([-1, 0]);
        this.setWidth(100);
        this.setHeight(25);
        let reg = props.register;
        let name = reg == 11 ? "cmp" : "r"+reg;
        this.text = name + " = " + props.value,
        this.register = props.register;
        this.value = props.value;
    }

    protected override draw(ctx: CanvasRenderingContext2D): void {
        ctx.fillStyle = "#BFBDB6";
        ctx.font = "24px Source Code Pro"
        ctx.textBaseline = "middle";
        ctx.fillText(this.text, -this.getWidth()/2, 2);
    }
}


export interface RegReadProps extends LayoutProps {
    text: string;
}

export class RegRead extends Layout {
    public text: string;

    public constructor(props?: RegReadProps) {
        super(props);
        this.offset([-1, 0]);
        this.setWidth(100);
        this.setHeight(25);
        this.text = props.text;
    }

    protected override draw(ctx: CanvasRenderingContext2D): void {
        ctx.fillStyle = "#BFBDB6";
        ctx.font = "24px Source Code Pro"
        ctx.textBaseline = "middle";
        ctx.fillText(this.text, -this.getWidth()/2, 2);
    }

    public* move_and_die(target: [number, number], duration: number) {
        yield* this.position(target, duration);
        this.remove();
    }
}
decorate(RegRead.prototype.move_and_die, threadable());


export class BasicSlot extends Layout {
    @initial(0.0)
    @property()
    public declare tag_opacity: Signal<number, this>;

    public tag: string;
    public instruction: Instruction | null;

    public overlay: ProgressOverlay | null;

    public constructor(props?: LayoutProps) {
        super(props);
        this.tag = "";
        this.instruction = null;
        this.overlay = null;
    }

    protected override draw(ctx: CanvasRenderingContext2D) {
        let rect = Rect.fromSizeCentered(this.size());

        ctx.fillStyle = "#2A303D";
        ctx.beginPath();
        drawRoundRect(ctx, rect, 5);
        ctx.fill();

        let w = this.size.x();
        ctx.save()
        ctx.fillStyle = "#7D8391";
        ctx.globalAlpha = this.tag_opacity();
        ctx.font = "bold 20px Source Code Pro"
        ctx.textBaseline = "middle";
        ctx.fillText(this.tag, -w/2 + 12, 2);
        ctx.restore();
    }
}


export class ProgressOverlay extends Layout {
    @initial(0.0)
    @property()
    public declare progress: Signal<number, this>;

    @initial(1.0)
    @property()
    public declare max_opacity: Signal<number, this>;

    public slot: BasicSlot;
    public task: any;

    public constructor(slot: BasicSlot) {
        super({});
        this.task = null;
        this.slot = slot;
        console.assert(slot.overlay === null);
        this.opacity(0);
        this.position(position_in_parent(slot));
        this.size(slot.size());
        slot.parent().parent().add(this);
        slot.overlay = this;
    }

    public* run(duration: number) {
        let fade = Math.min(0.1*duration, 0.1);
        this.opacity(0);
        this.progress(0);
        yield* all(
            this.opacity(this.max_opacity, fade),
            this.progress(1, duration, linear),
            delay(duration - fade, all(
                this.opacity(0, fade),
                this.slot.instruction?.wb?.opacity(1, fade),
            )),
        );

        this.slot.overlay = null;
        this.remove();
    }

    protected override draw(ctx: CanvasRenderingContext2D) {
        let rect = Rect.fromSizeCentered(this.size());

        ctx.fillStyle = "#232833";
        ctx.beginPath();
        drawRoundRect(ctx, rect, 5);
        ctx.fill();

        ctx.save();
        ctx.strokeStyle = "#97B0DB";
        ctx.lineWidth = 3;
        ctx.beginPath();
        ctx.arc(0, 0, 10, -Math.PI/2, -Math.PI/2 + 2*Math.PI*this.progress());
        ctx.stroke();
        ctx.restore();
    }
}
decorate(ProgressOverlay.prototype.run, threadable());



export interface ModuleProps extends LayoutProps {
    x: number;
    y: number;
    width:  number;
    height: number;
}


export class Decoder extends Layout {
    public slots: BasicSlot[];

    public constructor(props?: ModuleProps) {
        props.x += props.width/2;
        props.y += props.height/2;
        super(props);

        let y0 = -props.height/2 + 90;
        let width  = 250;
        let height = 50;
        this.slots = [
            new BasicSlot({ y: y0 + 0*60, width, height }),
            new BasicSlot({ y: y0 + 1*60, width, height }),
            new BasicSlot({ y: y0 + 2*60, width, height }),
            new BasicSlot({ y: y0 + 3*60, width, height }),
        ];
        this.add(this.slots);
    }

    public set_instructions(instructions: Instruction[]) {
        this.slots.forEach(s => {
            if(s.instruction) {
                s.instruction.remove();
                s.instruction = null;
            }
        });

        for(let i = 0; i < instructions.length; i += 1) {
            let instr = instructions[i];
            let instruction = new Instruction({
                asm: instr.asm, reads: instr.reads, writes: instr.writes,
                exec: instr.exec, unit: instr.unit,
                pc: instr.pc,
                jump_kind: instr.jump_kind, jump_target: instr.jump_target,
            });
            this.parent().add(instruction);

            let slot = this.slots[i];
            instruction.position(position_in_parent(slot));
            slot.instruction = instruction;
        }
    }

    public* decode(instructions: Instruction[], limit: number, duration: number) {
        this.set_instructions(instructions);
        this.slots.forEach(s => s.instruction?.opacity(0));

        let overlays = this.slots.slice(0, limit).map(s => new ProgressOverlay(s));

        yield* all(
            ...overlays.map(o => o.run(duration)),
            delay(0.4, all(
                ...this.slots.map(s => s.instruction?.opacity(1, 0.1)))),
        );
    }

    protected override draw(ctx: CanvasRenderingContext2D) {
        let rect = Rect.fromSizeCentered(this.size());
        draw_component("decoder", rect, ctx);

        super.draw(ctx);
    }
}
decorate(Decoder.prototype.decode, threadable());



export class Controller extends Layout {
    public slots: BasicSlot[];
    public in_flight: number;

    public constructor(props?: ModuleProps) {
        props.x += props.width/2;
        props.y += props.height/2;
        super(props);

        let y0 = -props.height/2 + 90;
        let width  = 400;
        let height = 50;
        let gap = 10;
        this.slots = [];
        for(let i = 0; i < 12; i += 1) {
            let slot = new BasicSlot({ y: y0 + i*(height + gap), width, height });
            slot.tag = String.fromCharCode(97 + i);
            slot.tag_opacity(1.0);
            this.slots.push(slot);
        }
        this.add(this.slots);

        this.in_flight = 0;
    }

    protected override draw(ctx: CanvasRenderingContext2D) {
        let rect = Rect.fromSizeCentered(this.size());
        draw_component("controller", rect, ctx);

        super.draw(ctx);
    }

    public* add_instructions(instructions: Instruction[]) {
        let n = instructions.length;

        let begin = this.slots.findIndex(s => !s.instruction);

        let anims = [];
        for(let i = 0; i < n; i += 1) {
            let slot = this.slots[begin + i];
            if(!slot) { break }

            let instruction = instructions[i];
            anims.push(instruction.position(position_in_parent(slot, -40), 0.4));
            slot.instruction = instruction;
        }
        yield* all(...anims);

        this.in_flight = 0;
        this.slots.forEach(s => {
            if(s.instruction) {
                this.in_flight += 1;
            }
        });
    }

    public* retire_instructions(count: number) {
        let top_slots = this.slots.slice(0, count);
        let bottom_slots = this.slots.slice(count);

        yield* all(...top_slots.map(s => s.instruction.opacity(0, 0.2)));
        top_slots.forEach(s => {
            s.instruction.remove();
            s.instruction = null;
        });

        let shift_up = count*60;
        let shift_down = (this.slots.length - count)*60;

        yield* all(
            ...top_slots.map(s => s.opacity(0, 0.2)),
            ...top_slots.map(s => s.position.y(s.position.y() + shift_down, 0.4)),
            ...top_slots.map(s => delay(0.2, s.opacity(1, 0.2))),
            ...bottom_slots.map(s => s.position.y(s.position.y() - shift_up, 0.4)),
            ...bottom_slots.filter(s => s.instruction)
                .map(s => {
                    let i = s.instruction;
                    return i.position.y(i.position.y() - shift_up, 0.4);
                }),
        );

        bottom_slots.push(...top_slots);
        this.slots = bottom_slots;

        this.in_flight = 0;
        this.slots.forEach(s => {
            if(s.instruction) {
                this.in_flight += 1;
            }
        });
    }
}
decorate(Controller.prototype.add_instructions, threadable());
decorate(Controller.prototype.retire_instructions, threadable());



export class Registers extends Layout {
    public registers: Register[];

    public constructor(props?: ModuleProps) {
        props.x += props.width/2;
        props.y += props.height/2;
        super(props);

        this.registers = [
            new Register({ name: "r0",  x: -65, y: -135 + 0*60 }),
            new Register({ name: "r1",  x:  65, y: -135 + 0*60 }),
            new Register({ name: "r2",  x: -65, y: -135 + 1*60 }),
            new Register({ name: "r3",  x:  65, y: -135 + 1*60 }),
            new Register({ name: "r4",  x: -65, y: -135 + 2*60 }),
            new Register({ name: "r5",  x:  65, y: -135 + 2*60 }),
            new Register({ name: "r6",  x: -65, y: -135 + 3*60 }),
            new Register({ name: "r7",  x:  65, y: -135 + 3*60 }),
            new Register({ name: "r8",  x: -65, y: -135 + 4*60 }),
            new Register({ name: "r9",  x:  65, y: -135 + 4*60 }),
            new Register({ name: "r10", x: -65, y: -135 + 5*60 }),
            new Register({ name: "cmp", x:  65, y: -135 + 5*60 }),
        ];
        this.registers.forEach(r => this.add(r));

        let cmp = this.registers[11];
        cmp.color = "#F29668";
        cmp.value = "eq";
    }

    protected override draw(ctx: CanvasRenderingContext2D) {
        let rect = Rect.fromSizeCentered(this.size());
        draw_component("registers", rect, ctx);
        super.draw(ctx);
    }
}


export interface RegisterProps extends LayoutProps {
    name: string;
}

export class Register extends Layout {
    public name:  string;
    public value: string;
    public color: string;

    public constructor(props?: RegisterProps) {
        super(props);
        this.setWidth(120);
        this.setHeight(50);
        this.name = props.name;
        this.value = "0";
        this.color = "#D2A6FF";
    }

    public* set_value(new_value: string) {
        yield* this.scale(1.2, 0.1);
        this.value = new_value;
        yield* this.scale(1, 0.1);
    }

    protected override draw(ctx: CanvasRenderingContext2D) {
        let rect = Rect.fromSizeCentered(this.size());

        ctx.fillStyle = "#2A303D";
        ctx.beginPath();
        drawRoundRect(ctx, rect, 5);
        ctx.fill();

        let w = this.size.x();
        ctx.save()
        ctx.textBaseline = "middle";

        ctx.fillStyle = "#7D8391";
        ctx.font = "bold 20px Source Code Pro"
        ctx.fillText(this.name, -w/2 + 12, 2);

        ctx.fillStyle = this.color;
        ctx.font = "20px Source Code Pro"
        ctx.textAlign = "end";
        ctx.fillText(this.value, w/2 - 18, 2);
        ctx.restore();
    }
}
decorate(Register.prototype.set_value, threadable());



export class MathUnit extends Layout {
    public slots: BasicSlot[];

    public constructor(props?: ModuleProps) {
        props.x += props.width/2;
        props.y += props.height/2;
        super(props);

        let y0 = -props.height/2 + 90;
        let width  = 400;
        let height = 50;
        let gap = 10;
        this.slots = [];
        for(let i = 0; i < 4; i += 1) {
            this.slots.push(new BasicSlot({ y: y0 + i*(height + gap), width, height }));
        }
        this.add(this.slots);
    }

    protected override draw(ctx: CanvasRenderingContext2D) {
        let rect = Rect.fromSizeCentered(this.size());
        draw_component("math unit", rect, ctx);
        super.draw(ctx);
    }
}



export class MemoryUnit extends Layout {
    public load_slots: BasicSlot[];
    public store_slots: BasicSlot[];

    public constructor(props?: ModuleProps) {
        props.x += props.width/2;
        props.y += props.height/2;
        super(props);

        let y0 = -props.height/2 + 90;
        let width  = 400;
        let height = 50;
        let gap = 10;
        y0 += 20;
        this.load_slots = [
            new BasicSlot({ y: y0 + 0*(height + gap), width, height }),
            new BasicSlot({ y: y0 + 1*(height + gap), width, height }),
            new BasicSlot({ y: y0 + 2*(height + gap), width, height }),
        ];
        y0 += 30;
        this.store_slots = [
            new BasicSlot({ y: y0 + 3*(height + gap), width, height }),
            new BasicSlot({ y: y0 + 4*(height + gap), width, height }),
        ];
        this.add(this.load_slots);
        this.add(this.store_slots);
    }

    protected override draw(ctx: CanvasRenderingContext2D) {
        let rect = Rect.fromSizeCentered(this.size());
        draw_component("memory unit", rect, ctx);
        super.draw(ctx);

        ctx.font = "italic 22px Source Code Pro"
        ctx.fillStyle = "#7D8391";

        let s = this.load_slots[0];
        let x = s.position.x() - s.size.x()/2;
        let y = s.position.y() - s.size.y()/2;
        x += 8;
        y -= 12;
        ctx.fillText("load", x, y);

        s = this.store_slots[0];
        y = s.position.y() - s.size.y()/2;
        y -= 12;
        ctx.fillText("store", x, y);
    }
}


export class CodeView extends Layout {
    public code: string[];
    public pc_to_line: number[];
    public program_index:  number;
    public decode_indices: number[];

    public constructor(props?: ModuleProps) {
        props.x += props.width/2;
        props.y += props.height/2;
        super(props);
        this.code = [];
        this.pc_to_line = [0];
        this.program_index = 0;
        this.decode_indices = [];
    }

    public set_code(code: string[]) {
        let pc_to_line: number[] = [];
        code.forEach((line, index) => {
            line = line.trim();
            if(line.length > 0 && !line.endsWith(":")) {
                pc_to_line.push(index);
            }
        });
        pc_to_line.push(code.length);
        this.code = code;
        this.pc_to_line = pc_to_line;

        let y = this.position.y() - this.getHeight()/2;
        let h = 110 + code.length*26;
        this.position.y(y + h/2);
        this.setHeight(h);
    }

    public set_counters(pc: number, dc: number, decode_limit: number) {
        this.program_index = this.pc_to_line[pc];

        this.decode_indices.length = 0;
        let dc_end = Math.min(this.pc_to_line.length - 1, dc + decode_limit);
        for(let i = dc; i < dc_end; i += 1) {
            this.decode_indices.push(this.pc_to_line[i]);
        }
        if(this.decode_indices.length == 0) {
            this.decode_indices.push(this.code.length);
        }
    }

    protected override draw(ctx: CanvasRenderingContext2D) {
        let rect = Rect.fromSizeCentered(this.size());
        draw_component("assembly code", rect, ctx);

        let x0 = -this.getWidth()/2 + 36;
        let y0 = -this.getHeight()/2 + 70;
        let gap = 26;

        ctx.fillStyle = "#BFBDB6";
        ctx.font = "22px Source Code Pro"
        ctx.textBaseline = "top";
        this.code.forEach((line, index) => {
            ctx.fillText(line, x0, y0 + gap*index + 2);
        });

        ctx.fillStyle = "#7D8391";
        ctx.beginPath();
        for(const di of this.decode_indices) {
            ctx.ellipse(x0 + 19, y0 + gap*(di + 0.5), 3, 3, 0, 0, 2*Math.PI);
            ctx.closePath();
        }
        ctx.fill();

        let pi = this.program_index;
        ctx.fillStyle = "yellow";
        ctx.strokeStyle = "#232833";
        ctx.lineWidth = 1;
        ctx.beginPath(); {
            let x = x0 - 5;
            let y = y0 + gap*(pi + 0.5);
            ctx.moveTo(x, y);
            ctx.lineTo(x, y + 4);
            ctx.lineTo(x + 7, y + 4);
            ctx.lineTo(x + 7, y + 10);
            ctx.lineTo(x + 17, y);
            ctx.lineTo(x + 7, y - 10);
            ctx.lineTo(x + 7, y - 4);
            ctx.lineTo(x, y - 4);
            ctx.closePath();
        }
        ctx.fill();
        ctx.stroke();

        super.draw(ctx);
    }
}


import {Layout, LayoutProps} from "@motion-canvas/2d/lib/components"
import {Rect, Vector2} from "@motion-canvas/core/lib/types";
import {property} from "@motion-canvas/2d/lib/decorators"
import {all, sequence} from "@motion-canvas/core/lib/flow";
import {Signal, SignalValue} from "@motion-canvas/core/lib/utils";
import {drawRect} from "@motion-canvas/2d/lib/utils";
import {easeOutCubic, linear} from "@motion-canvas/core/lib/tweening";
import {decorate, threadable} from "@motion-canvas/core/lib/decorators"


const ITEM_WIDTH  = 300;
const ITEM_HEIGHT = 110;
const ITEM_MARGIN = 18;
const ITEM_GAP = 6;


export class Stack extends Layout {
    items: StackItem[];
    base: number;

    public constructor(props?: LayoutProps) {
        super(props);
        this.items = [];
        this.setWidth(400);
        this.setHeight(500);
        this.base = 250 - ITEM_HEIGHT/2 - 30;
    }

    protected override draw(ctx: CanvasRenderingContext2D) {
        ctx.fillStyle = "#FFB454";

        let w = this.getWidth();
        let h = this.getHeight();
        ctx.beginPath();
        ctx.rect(-w/2, h/2 - 10, w, 10);
        ctx.fill();

        super.draw(ctx);
    }

    item_pos(index: number): number {
        return this.base - index*(ITEM_HEIGHT + ITEM_GAP);
    }

    idx_rev(index: number): StackItem {
        return this.items[this.items.length - 1 - index];
    }

    public new_item(value: string): StackItem {
        let item = new StackItem({value});
        this.items.push(item);
        this.add(item);

        item.position.y(this.item_pos(this.items.length - 1));
        return item;
    }

    public remove_item(idx: number): StackItem {
        let result = this.items[idx];
        this.items.splice(idx, 1);
        return result;
    }

    public* remove_item_fade(idx: number) {
        let item = this.remove_item(idx);
        yield* item.opacity(0, 0.2);
        item.remove();
    }

    public* replace_items(new_values: string[]) {
        let items = this.items;
        this.items = [];
        yield* all(
            ...items.map(item => item.opacity(0, 0.2)),
            sequence(0.1, ...new_values.map(value => this.drop_new(value))),
        )
    }

    public* highlight_item(idx: number) {
        let item = this.items[idx];
        yield* item.scale(1.15, 0.15);
        yield* item.scale(1.00, 0.15);
    }

    public clear() {
        for(const item of this.items) {
            item.remove();
        }
        this.items.length = 0;
    }

    public* drop_new(value: string) {
        let item = this.new_item(value);
        let y = item.position.y();
        item.position.y(y - 800);
        yield* item.position.y(y, 0.3, linear);
    }

    public* load(index: number) {
        let original = this.items[index];
        let item = this.new_item(original.value());

        let y0 = original.position.y();
        let y1 = item.position.y();
        item.position.y(y0);
        yield* item.position.x(ITEM_WIDTH + 30, 0.2);
        yield* item.position.y(y1, 0.2);
        yield* item.position.x(0, 0.2);
    }

    public* store(index: number) {
        let target = this.items[index];
        let item = this.items.pop();

        let y1 = target.position.y();
        yield* item.position.x(ITEM_WIDTH + 30, 0.2);
        yield* item.position.y(y1, 0.2);
        yield* item.position.x(0, 0.2);

        this.items[index] = item;
        target.remove();
    }

    public* dup() {
        let src = this.idx_rev(0);
        let item = this.new_item(src.value());

        let y = item.position.y();
        item.position.y(src.position.y());
        yield* item.position.y(y, 0.2);
    }

    public* rot() {
        let a = this.idx_rev(2);
        let b = this.idx_rev(1);
        let c = this.idx_rev(0);

        let bot = a.position.y();
        let mid = b.position.y();
        let top = c.position.y();

        yield* a.position.x(ITEM_WIDTH + 30, 0.2);
        yield* all(
            a.position.y(top, 0.2),
            b.position.y(bot, 0.2),
            c.position.y(mid, 0.2),
        );
        yield* a.position.x(0, 0.2);

        this.items.length = this.items.length - 3;
        this.items.push(b);
        this.items.push(c);
        this.items.push(a);
    }

    public* bin_op_stack(result: string) {
        let b = this.idx_rev(0);
        let a = this.idx_rev(1);

        let y = b.position.y();

        yield* b.position(new Vector2( 200, y - 30), 0.2, easeOutCubic);
        yield* a.position(new Vector2(-200, y - 30), 0.2, easeOutCubic);
        yield* all(
            a.position.x(0, 0.2),
            b.position.x(0, 0.2),
        );

        let bi = this.items.pop();
        bi.remove();
        let ai = this.items.pop();
        ai.remove();

        let r = this.new_item(result);
        let ry = r.position.y();
        r.position.y(y - 30);
        yield* r.position.y(ry, 0.2);
    }

    make_clone(item: StackItem): StackItem {
        let r = new StackItem({value: item.value()});
        this.add(r);
        r.position(item.position());
        return r;
    }

    public* copy(dst: number, src: number) {
        let d = this.items[dst];
        let s = this.items[src];

        let sc = this.make_clone(s);

        yield* sc.position.x(ITEM_WIDTH + 30, 0.2);
        yield* sc.position.y(d.position.y(), 0.2);
        yield* sc.position.x(0, 0.2);

        sc.remove();
        d.value(s.value());
    }

    public* bin_op_reg(dst: number, src1: number, src2: number, result: string, side: number = 1) {
        if(dst == this.items.length) {
            let top = this.new_item("");
            top.opacity(0);
        }

        let d  = this.items[dst];
        let s1 = this.items[src1];
        let s2 = this.items[src2];

        let s1c = this.make_clone(s1);
        let s2c = this.make_clone(s2);

        yield* s1c.position.x(side*(ITEM_WIDTH + 30), 0.2);
        if(src1 == src2) { yield* s1c.position.y(s1c.position.y() - s1c.size.y()*1.1, 0.2); }

        yield* s2c.position.x(side*(ITEM_WIDTH + 30), 0.2);

        let y = (s1c.position.y() + s2c.position.y()) * 0.5;
        yield* all(
            s1c.position.y(y, 0.2),
            s2c.position.y(y, 0.2),
        );

        s2c.remove();
        s1c.value(result);

        yield* s1c.position.y(d.position.y(), 0.2);
        yield* s1c.position.x(0, 0.2);

        s1c.remove();
        d.value(result);
        d.opacity(1);
    }
}

decorate(Stack.prototype.remove_item_fade, threadable());
decorate(Stack.prototype.replace_items, threadable());
decorate(Stack.prototype.highlight_item, threadable());
decorate(Stack.prototype.drop_new, threadable());
decorate(Stack.prototype.load, threadable());
decorate(Stack.prototype.store, threadable());
decorate(Stack.prototype.dup, threadable());
decorate(Stack.prototype.rot, threadable());
decorate(Stack.prototype.bin_op_stack, threadable());
decorate(Stack.prototype.copy, threadable());
decorate(Stack.prototype.bin_op_reg, threadable());


export interface StackItemProps extends LayoutProps {
    value: SignalValue<string>;
}

export class StackItem extends Layout {
    @property()
    public declare value: Signal<string, this>;

    @property()
    public declare bg: Signal<string, this>;

    public constructor(props?: StackItemProps) {
        super(props);

        this.setWidth(ITEM_WIDTH);
        this.setHeight(ITEM_HEIGHT);
        this.value(props.value);
        this.bg("#0D1017");
    }

    protected override draw(ctx: CanvasRenderingContext2D) {
        ctx.fillStyle = this.bg();
        ctx.strokeStyle = "#FFB454";
        ctx.lineWidth = 10;

        const rect = Rect.fromSizeCentered(this.size().sub(new Vector2(ITEM_MARGIN)));
        ctx.beginPath();
        drawRect(ctx, rect);
        ctx.fill();
        ctx.stroke();

        ctx.fillStyle = "#D2A6FF";
        ctx.textAlign = "center";
        ctx.font = "52px Source Code Pro";
        ctx.fillText(this.value(), 0, 15);
    }
}



const SPEEEEEED: bool = 1==1;


pub mod reg {

    #[derive(Clone, Copy, Debug)]
    #[repr(align(4))]
    pub enum Instruction {
        LoadInt     { dst: u8, value: i16 },
        Copy        { dst: u8, src: u8 },
        Add         { dst: u8, src1: u8, src2: u8 },
        Sub         { dst: u8, src1: u8, src2: u8 },
        Mul         { dst: u8, src1: u8, src2: u8 },
        Jump        { target: u8 },
        SetCounter  { src: u8 },
        GetCounter  { dst: u8 },
        Loop        { target: u8 },
        LoopLe      { target: u8, src1: u8, src2: u8 },
        Return      { src: u8 },
    }

    pub struct Vm {
        registers: Vec<f64>,
    }

    struct State<'a> {
        vm: &'a mut Vm,
        code: &'a [Instruction],
        pc: usize,
        pcp: *const Instruction,
        counter: u32,
    }

    impl Vm {
        pub fn new() -> Self {
            Vm { registers: vec![0.0; 256] }
        }

        #[inline(never)]
        pub fn run(&mut self, code: &[Instruction], args: &[f64]) -> f64 {
            let mut s = State {
                vm: self,
                code,
                pc: 0,
                pcp: core::ptr::null(),
                counter: 0,
            };

            s.jump(0);
            for (i, arg) in args.iter().enumerate() {
                s.vm.registers[i] = *arg;
            }

            loop {
                let instr = s.next_instr();

                use Instruction::*;
                match instr {
                    LoadInt { dst, value } => {
                        *s.reg(dst) = value as f64;
                    }

                    Copy { dst, src } => {
                        *s.reg(dst) = *s.reg(src);
                    }

                    Add { dst, src1, src2 } => {
                        *s.reg(dst) = *s.reg(src1) + *s.reg(src2);
                    }

                    Sub { dst, src1, src2 } => {
                        *s.reg(dst) = *s.reg(src1) - *s.reg(src2);
                    }

                    Mul { dst, src1, src2 } => {
                        *s.reg(dst) = *s.reg(src1) * *s.reg(src2);
                    }

                    Jump { target } => {
                        s.jump(target);
                    }

                    SetCounter { src } => {
                        s.counter = *s.reg(src) as u32;
                    }

                    GetCounter { dst } => {
                        *s.reg(dst) = s.counter as f64;
                    }

                    Loop { target } => {
                        if s.counter > 0 {
                            s.counter -= 1;
                            s.jump(target);
                        }
                    }

                    LoopLe { target, src1, src2 } => {
                        let a = *s.reg(src1);
                        let b = *s.reg(src2);
                        if a <= b && s.counter > 0 {
                            s.counter -= 1;
                            s.jump(target);
                        }
                    }

                    Return { src } => {
                        let result = *s.reg(src);
                        return result;
                    }
                }
            }
        }
    }

    impl<'a> State<'a> {
        #[inline(always)]
        fn next_instr(&mut self) -> Instruction {
            if super::SPEEEEEED {
                unsafe {
                    let result = *self.pcp;
                    self.pcp = self.pcp.add(1);
                    result
                }
            }
            else {
                let result = self.code[self.pc];
                self.pc += 1;
                result
            }
        }

        #[inline(always)]
        fn jump(&mut self, target: u8) {
            if super::SPEEEEEED {
                unsafe {
                    self.pcp = self.code.as_ptr().add(target as usize);
                }
            }
            else {
                self.pc = target as usize;
            }
        }

        #[inline(always)]
        fn reg(&mut self, index: u8) -> &mut f64 {
            if super::SPEEEEEED {
                unsafe { self.vm.registers.get_unchecked_mut(index as usize) }
            }
            else {
                &mut self.vm.registers[index as usize]
            }
        }
    }


    pub const FIB: &[Instruction] = { use Instruction::*; &[
        SetCounter { src: 0 },
        LoadInt { dst: 1, value: 0 },
        LoadInt { dst: 2, value: 1 },
        Jump { target: 7 },
        Add { dst: 3, src1: 1, src2: 2 },
        Copy { dst: 1, src: 2 },
        Copy { dst: 2, src: 3 },
        Loop { target: 4 },
        Return { src: 1 },
    ]};


    pub const MANDEL: &[Instruction] = { use Instruction::*; let (x0, y0, n, x, y, t0, t1) = (0, 1, 2, 3, 4, 5, 6); &[
        LoadInt { dst: x, value: 0 },
        LoadInt { dst: y, value: 0 },

        // 2
        SetCounter { src: n },

        Jump { target: 13 },

        // 4
        // let xtemp = x*x - y*y + x0;
        Mul { dst: t0, src1: x, src2: x },
        Mul { dst: t1, src1: y, src2: y },
        Sub { dst: t0, src1: t0, src2: t1 },
        Add { dst: t0, src1: t0, src2: x0 },
        // y = x*y*2.0 + y0;
        Mul { dst: y, src1: x, src2: y },
        LoadInt { dst: t1, value: 2 },
        Mul { dst: y, src1: y, src2: t1 },
        Add { dst: y, src1: y, src2: y0 },
        // x = xtemp
        Copy { dst: x, src: t0 },

        // check x*x + y*y <= 2*2
        // 13
        Mul { dst: t0, src1: x, src2: x },
        Mul { dst: t1, src1: y, src2: y },
        Add { dst: t0, src1: t0, src2: t1 },
        LoadInt { dst: t1, value: 4 },
        LoopLe { target: 4, src1: t0, src2: t1 },

        // 18
        GetCounter { dst: t1 },
        Sub { dst: t0, src1: n, src2: t1 },
        Return { src: t0 },
    ]};


    pub const ADD_CHAIN: &[Instruction] = { use Instruction::*; &[
        Add { dst: 0, src1: 0, src2:  1 },
        Add { dst: 0, src1: 0, src2:  2 },
        Add { dst: 0, src1: 0, src2:  3 },
        Add { dst: 0, src1: 0, src2:  4 },
        Add { dst: 0, src1: 0, src2:  5 },
        Add { dst: 0, src1: 0, src2:  6 },
        Add { dst: 0, src1: 0, src2:  7 },
        Add { dst: 0, src1: 0, src2:  8 },
        Add { dst: 0, src1: 0, src2:  9 },
        Add { dst: 0, src1: 0, src2: 10 },
        Add { dst: 0, src1: 0, src2: 11 },
        Add { dst: 0, src1: 0, src2: 12 },
        Add { dst: 0, src1: 0, src2: 13 },
        Add { dst: 0, src1: 0, src2: 14 },
        Add { dst: 0, src1: 0, src2: 15 },
        Return { src: 0 },
    ]};

    pub const ADD_PAIRS: &[Instruction] = { use Instruction::*; &[
        Add { dst:  0, src1:  0, src2:  1 },
        Add { dst:  2, src1:  2, src2:  3 },
        Add { dst:  4, src1:  4, src2:  5 },
        Add { dst:  6, src1:  6, src2:  7 },
        Add { dst:  8, src1:  8, src2:  9 },
        Add { dst: 10, src1: 10, src2: 11 },
        Add { dst: 12, src1: 12, src2: 13 },
        Add { dst: 14, src1: 14, src2: 15 },
        Add { dst:  0, src1:  0, src2:  2 },
        Add { dst:  4, src1:  4, src2:  6 },
        Add { dst:  8, src1:  8, src2: 10 },
        Add { dst: 12, src1: 12, src2: 14 },
        Add { dst:  0, src1:  0, src2:  4 },
        Add { dst:  8, src1:  8, src2: 12 },
        Add { dst:  0, src1:  0, src2:  8 },
        Return { src: 0 },
    ]};
}


pub mod stack {

    #[derive(Clone, Copy, Debug)]
    #[repr(align(2))]
    pub enum Instruction {
        Load         { src: u8 },
        Store        { dst: u8 },
        LoadInt      { value: i8 },
        Add,
        Sub,
        Mul,
        Pop,
        Dup,
        Rot,
        Swap,
        Jump         { target: u8 },
        SetCounter,
        GetCounter,
        Loop         { target: u8 },
        LoopLe       { target: u8 },
        Return,
        Nop,
    }

    pub struct Vm {
        stack: Vec<f64>,
    }

    struct State<'a> {
        vm: &'a mut Vm,
        code: &'a [Instruction],
        pc: usize,
        pcp: *const Instruction,
        counter: u32,
        base: *mut f64,
        top:  *mut f64,
    }

    impl Vm {
        pub fn new() -> Self {
            Vm { stack: Vec::with_capacity(256) }
        }

        #[inline(never)]
        pub fn run(&mut self, code: &[Instruction], args: &[f64]) -> f64 {
            let base = self.stack.as_mut_ptr();
            let base = ((base as usize + 63) / 64 * 64) as *mut f64;

            let mut s = State {
                code,
                pc: 0,
                pcp: core::ptr::null(),
                base,
                top: base,
                counter: 0,
                vm: self,
            };

            s.jump(0);
            for arg in args {
                s.push(*arg);
            }

            loop {
                let instr = s.next_instr();

                use Instruction::*;
                match instr {
                    Load { src } => {
                        let value = *s.get(src);
                        s.push(value);
                    }

                    Store { dst } => {
                        let value = s.pop();
                        *s.get(dst) = value;
                    }

                    LoadInt { value } => {
                        s.push(value as f64);
                    }

                    Add => {
                        *s.get_top(1) = *s.get_top(1) + *s.get_top(0);
                        s.pop();
                    }

                    Sub => {
                        *s.get_top(1) = *s.get_top(1) - *s.get_top(0);
                        s.pop();
                    }

                    Mul => {
                        *s.get_top(1) = *s.get_top(1) * *s.get_top(0);
                        s.pop();
                    }

                    Pop => {
                        s.pop();
                    }

                    Dup => {
                        let value = *s.get_top(0);
                        s.push(value);
                    }

                    Rot => {
                        let a = *s.get_top(2);
                        *s.get_top(2) = *s.get_top(1);
                        *s.get_top(1) = *s.get_top(0);
                        *s.get_top(0) = a;
                    }

                    Swap => {
                        let a = *s.get_top(0);
                        *s.get_top(0) = *s.get_top(1);
                        *s.get_top(1) = a;
                    }

                    Jump { target } => {
                        s.jump(target);
                    }

                    SetCounter => {
                        let value = s.pop();
                        s.counter = value as u32;
                    }

                    GetCounter => {
                        s.push(s.counter as f64);
                    }

                    Loop { target } => {
                        if s.counter > 0 {
                            s.counter -= 1;
                            s.jump(target);
                        }
                    }

                    LoopLe { target } => {
                        let b = s.pop();
                        let a = s.pop();

                        if a <= b && s.counter > 0 {
                            s.counter -= 1;
                            s.jump(target);
                        }
                    }

                    Return => {
                        let result = s.pop();
                        s.clear();
                        return result;
                    }

                    Nop => {}
                }
            }
        }
    }

    impl<'a> State<'a> {
        #[inline(always)]
        fn next_instr(&mut self) -> Instruction {
            if super::SPEEEEEED {
                unsafe {
                    let result = *self.pcp;
                    self.pcp = self.pcp.add(1);
                    result
                }
            }
            else {
                let result = self.code[self.pc];
                self.pc += 1;
                result
            }
        }

        #[inline(always)]
        fn jump(&mut self, target: u8) {
            if super::SPEEEEEED {
                unsafe {
                    self.pcp = self.code.as_ptr().add(target as usize);
                }
            }
            else {
                self.pc = target as usize;
            }
        }

        #[inline(always)]
        fn get(&mut self, index: u8) -> &mut f64 {
            if super::SPEEEEEED {
                unsafe {
                    &mut *self.base.add(index as usize)
                }
            }
            else {
                &mut self.vm.stack[index as usize]
            }
        }

        #[inline(always)]
        fn get_top(&mut self, index: u8) -> &mut f64 {
            if super::SPEEEEEED {
                unsafe {
                    &mut *self.top.sub(1).sub(index as usize)
                }
            }
            else {
                let len = self.vm.stack.len();
                &mut self.vm.stack[len - 1 - index as usize]
            }
        }

        #[inline(always)]
        fn push(&mut self, value: f64) {
            if super::SPEEEEEED {
                unsafe {
                    *self.top = value;
                    self.top = self.top.add(1);
                }
            }
            else {
                self.vm.stack.push(value)
            }
        }

        #[inline(always)]
        fn pop(&mut self) -> f64 {
            if super::SPEEEEEED {
                unsafe {
                    self.top = self.top.sub(1);
                    *self.top
                }
            }
            else {
                self.vm.stack.pop().unwrap()
            }
        }

        #[inline(always)]
        fn clear(&mut self) {
            if super::SPEEEEEED {
                self.top = self.base;
            }
            else {
                self.vm.stack.clear();
            }
        }
    }


    pub const FIB_SMART: &[Instruction] = { use Instruction::*; &[
        SetCounter,
        LoadInt { value: 0 },
        LoadInt { value: 1 },
        Jump { target: 7 },
        Dup,
        Rot,
        Add,
        Loop { target: 4 },
        Pop,
        Return,
    ]};

    pub const FIB_NAIVE: &[Instruction] = { use Instruction::*; &[
        SetCounter,
        LoadInt { value: 0 },
        LoadInt { value: 1 },
        Jump { target: 10 },
        // 4
        Load { src: 0 },
        Load { src: 1 },
        Add,
        Load { src: 1 },
        Store { dst: 0 },
        Store { dst: 1 },
        // 10
        Loop { target: 4 },
        Pop,
        Return,
    ]};


    pub const MANDEL_SMART: &[Instruction] = { use Instruction::*; let (x0, y0, n, x, y) = (0, 1, 2, 3, 4); &[
        Load { src: n },
        SetCounter,

        LoadInt { value: 0 },
        LoadInt { value: 0 },

        Jump { target: 21 },

        // 5
        // let xtemp = x*x - y*y + x0;
        Load { src: x },
        Dup,
        Mul,
        Load { src: y },
        Dup,
        Mul,
        Sub,
        Load { src: x0 },
        Add,

        // 14
        // stack: x0, y0, n, x, y, xtemp
        Swap,
        // stack: x0, y0, n, x, xtemp, y
        Rot,
        // stack: x0, y0, n, xtemp, y, x

        // 16
        // y = x*y*2.0 + y0;
        Mul,
        LoadInt { value: 2 },
        Mul,
        Load { src: y0 },
        Add,

        // check x*x + y*y <= 2*2
        // 21
        Load { src: x },
        Dup,
        Mul,
        Load { src: y },
        Dup,
        Mul,
        Add,
        LoadInt { value: 4 },
        LoopLe { target: 5 },


        // 30
        Load { src: n },
        GetCounter,
        Sub,
        Return,
    ]};

    pub const MANDEL_NAIVE: &[Instruction] = { use Instruction::*; let (x0, y0, n, x, y, xtemp) = (0, 1, 2, 3, 4, 5); &[
        Load { src: n },
        SetCounter,

        LoadInt { value: 0 },
        LoadInt { value: 0 },
        LoadInt { value: 0 },
        // stack: x0, y0, n, x, y, xtemp

        Jump { target: 26 },

        // 6
        // let xtemp = x*x - y*y + x0;
        Load { src: x },
        Load { src: x },
        Mul,
        Load { src: y },
        Load { src: y },
        Mul,
        Sub,
        Load { src: x0 },
        Add,
        Store { dst: xtemp },

        // 16
        // y = x*y*2.0 + y0;
        Load { src: x },
        Load { src: y },
        Mul,
        LoadInt { value: 2 },
        Mul,
        Load { src: y0 },
        Add,
        Store { dst: y },

        // 24
        // x = xtemp
        Load { src: xtemp },
        Store { dst: x },

        // check x*x + y*y <= 2*2
        // 26
        Load { src: x },
        Load { src: x },
        Mul,
        Load { src: y },
        Load { src: y },
        Mul,
        Add,
        LoadInt { value: 4 },
        LoopLe { target: 6 },


        // 35
        Load { src: n },
        GetCounter,
        Sub,
        Return,
    ]};


    pub const MANDEL_SMART_NOPS_SLOW: &[Instruction] = { use Instruction::*; let (x0, y0, n, x, y) = (0, 1, 2, 3, 4); &[
        Load { src: n },
        SetCounter,

        LoadInt { value: 0 },
        LoadInt { value: 0 },

        Jump { target: 21 + 3 },

        // 5
        // let xtemp = x*x - y*y + x0;
        Load { src: x },
        Dup,
        Mul,
        Load { src: y },
        Nop, Nop, Nop,
        Dup,
        Mul,
        Sub,
        Load { src: x0 },
        Add,

        // 14 + 3
        // stack: x0, y0, n, x, y, xtemp
        Swap,
        // stack: x0, y0, n, x, xtemp, y
        Rot,
        // stack: x0, y0, n, xtemp, y, x

        // 16 + 3
        // y = x*y*2.0 + y0;
        Mul,
        LoadInt { value: 2 },
        Mul,
        Load { src: y0 },
        Add,

        // check x*x + y*y <= 2*2
        // 21 + 3
        Load { src: x },
        Dup,
        Mul,
        Load { src: y },
        Dup,
        Mul,
        Add,
        LoadInt { value: 4 },
        LoopLe { target: 5 },


        // 30 + 3
        Load { src: n },
        GetCounter,
        Sub,
        Return,
    ]};

    pub const MANDEL_SMART_NOPS_SAME: &[Instruction] = { use Instruction::*; let (x0, y0, n, x, y) = (0, 1, 2, 3, 4); &[
        Load { src: n },
        SetCounter,

        LoadInt { value: 0 },
        LoadInt { value: 0 },

        Jump { target: 21 + 3 },

        // 5
        // let xtemp = x*x - y*y + x0;
        Load { src: x },
        Dup,
        Mul,
        Load { src: y },
        Dup,
        Mul,
        Sub,
        Load { src: x0 },
        Add,

        // 14
        // stack: x0, y0, n, x, y, xtemp
        Swap,
        // stack: x0, y0, n, x, xtemp, y
        Rot,
        // stack: x0, y0, n, xtemp, y, x
        Nop, Nop, Nop,

        // 16 + 3
        // y = x*y*2.0 + y0;
        Mul,
        LoadInt { value: 2 },
        Mul,
        Load { src: y0 },
        Add,

        // check x*x + y*y <= 2*2
        // 21 + 3
        Load { src: x },
        Dup,
        Mul,
        Load { src: y },
        Dup,
        Mul,
        Add,
        LoadInt { value: 4 },
        LoopLe { target: 5 },


        // 30 + 3
        Load { src: n },
        GetCounter,
        Sub,
        Return,
    ]};

    pub const MANDEL_SMART_NO_DUP: &[Instruction] = { use Instruction::*; let (x0, y0, n, x, y) = (0, 1, 2, 3, 4); &[
        Load { src: n },
        SetCounter,

        LoadInt { value: 0 },
        LoadInt { value: 0 },

        Jump { target: 21 },

        // 5
        // let xtemp = x*x - y*y + x0;
        Load { src: x },
        Load { src: x },
        Mul,
        Load { src: y },
        Load { src: y },
        Mul,
        Sub,
        Load { src: x0 },
        Add,

        // 14
        // stack: x0, y0, n, x, y, xtemp
        Swap,
        // stack: x0, y0, n, x, xtemp, y
        Rot,
        // stack: x0, y0, n, xtemp, y, x

        // 16
        // y = x*y*2.0 + y0;
        Mul,
        LoadInt { value: 2 },
        Mul,
        Load { src: y0 },
        Add,

        // check x*x + y*y <= 2*2
        // 21
        Load { src: x },
        Load { src: x },
        Mul,
        Load { src: y },
        Load { src: y },
        Mul,
        Add,
        LoadInt { value: 4 },
        LoopLe { target: 5 },


        // 30
        Load { src: n },
        GetCounter,
        Sub,
        Return,
    ]};
}



#[inline(never)]
pub fn fib(n: f64) -> f64 {
    let mut a = 0.0;
    let mut b = 1.0;
    for _ in 0..n as u32 {
        (a, b) = (b, a+b);
    }
    a
}

#[inline(never)]
pub fn mandel(x0: f64, y0: f64, limit: f64) -> f64 {
    let mut x = 0.0;
    let mut y = 0.0;
    let mut i = 0;
    while x*x + y*y <= 4.0 && i < limit as u32 {
        let xtemp = x*x - y*y + x0;
        y = x*y*2.0 + y0;
        x = xtemp;
        i += 1;
    }
    return i as f64;
}


#[cfg(test)]
mod tests {
    use super::*;

    fn test_fib<F: FnMut(f64) -> f64>(mut f: F) {
        for i in 0..1000 {
            assert_eq!(fib(i as f64), f(i as f64));
        }
    }

    fn test_mandel<F: FnMut(f64, f64, f64) -> f64>(mut f: F) {
        let limit = 10000.0;
        for (x, y) in [(0.0, 0.0), (1.0, 1.0), (0.239, -0.981), (-0.648, 0.129), (-0.812, -0.021), (0.687, 0.387), (-0.950786716408015, 0.7448839625558739), (-0.41795412221209083, 0.3891104473889062), (-0.30454550287093674, 0.3662807658368674), (-0.38954007305330274, 0.6099259700840349), (0.07342221713511665, -0.8427951153696895), (-0.11064854851199213, -0.926639764442595), (-0.7596206479056045, 0.048924374342336874), (-0.49581061588344877, -0.12784717276741908), (0.6437608077269181, 0.4495234224170379), (-0.35902711244632446, -0.27337416802046066), (0.46119571367001844, 0.6369625077084937), (-0.8337757474334115, 0.30803404049145233), (0.7567477274895906, 0.7425733436366662), (-0.05666196234341658, -0.7153313926145124), (-0.1712571411242514, -0.7293648513842237), (-0.18029887642403808, 0.9337582385391014), (0.15025609901930093, -0.2562979396981093), (-0.8740713363980595, -0.3026974138730323), (0.734282601227346, 0.44824566299076163), (-0.6450519022478793, 0.8973350411345271), (-0.12951887416193664, -0.9675733335426315), (-0.8496395992141339, -0.45856363991261984), (-0.05006755838234733, 0.08815703999052182), (-0.0819349209740825, 0.392640373018607), (0.38818643312858203, 0.017802320175963837), (-0.5804782726092039, -0.8114288891880093), (0.21174287430534067, -0.9779934844282805), (0.48485403501831414, -0.32072813452059323), (0.11816231997762872, 0.9901345882876014), (-0.32987818756568665, -0.790041653198585), (-0.751877131490938, 0.7158915912037904), (-0.27952689192889313, -0.990237696155138), (0.27114775350097875, -0.4153077703274628), (0.058473091300088376, -0.9049579783144228), (0.4753437616586973, 0.14657146392303133), (-0.1745580736849639, 0.4775850047960255), (-0.7249298752680955, -0.9641325799319125), (0.12624851616576604, -0.44348534069122003), (-0.973539445295293, 0.832103245702291), (-0.6743590940233655, 0.9387520845818487), (-0.16662572477960436, -0.900765313197615), (-0.5912590709584258, -0.5102679290062448), (0.8503235254831967, -0.33485304549647044), (0.08977059985414226, -0.5524679311716958), (0.9092630698742039, 0.651495043750099), (0.06998115038035468, -0.1554190026368456), (-0.7399797760579021, -0.006137159260573455), (-0.2757987315085746, -0.537347842353135), (-0.30389255510677926, 0.542418339351705), (0.05627319272720821, 0.6292518136210365), (-0.6203030673427403, 0.7900418292632594), (-0.7909573091413067, -0.396979443474341), (0.24824386339155158, 0.3388202675220937), (-0.0513868729451854, 0.17817438126698026), (0.9462416973938339, 0.8676690260189799), (-0.1139783605452167, 0.0038380988757436008), (-0.23121289597591965, -0.47531036160660034), (0.5828474701609332, -0.7839471493639378), (0.9770948454943438, -0.4683608606405316), (0.47740801560355584, -0.2883965096412553), (0.34009761202328015, 0.6546012541731507), (-0.4924202923168335, 0.7856942391002899), (0.6230983780135113, -0.19678697882493723), (-0.140144286728481, 0.9028641035510641), (0.02303038331068996, -0.3161713074917263), (-0.6435921111392493, 0.6404964303777942), (-0.7066891000784838, 0.45541585282911434), (-0.24137952675687369, -0.7447054863100193), (0.19991366412404066, -0.23703689667205108), (-0.9570794031819638, -0.9884389720144522), (-0.6797569127393104, 0.2082269559276082), (0.769667914938132, 0.3787028736840006), (0.5848065627210588, 0.21127658721982723), (0.4844247997855926, 0.7800864760973392), (0.4583131215187135, 0.8734790105541512), (-0.3952706913645596, 0.633963083692719), (-0.9486119273737694, 0.7587301881016015), (-0.46070283891613584, 0.5041607377115895), (0.995619007269551, 0.3803371649267848), (-0.2797833504752094, 0.9908037027712009), (-0.9167995403904445, -0.807580777212757), (-0.18027266265314745, -0.851839465543321), (0.6128426674227088, -0.09875532852024493), (0.011860446602936614, -0.8681653691688065), (-0.7107796408969822, 0.02980060781663707), (-0.06413027397564885, -0.13037851125697242), (0.11553944723254239, -0.4890274430377839), (0.3479886739255793, -0.8699677560350583), (0.38333167839236126, 0.4218707211211812), (0.638723023074957, -0.9683866946207558), (0.46354649116031754, 0.07563976922668769), (-0.06351773294699603, 0.8144404275239079), (0.5119372264249904, -0.9096216627089817), (-0.09883485075616494, 0.3297820865529715), (0.10358292605721098, -0.36692635243618077), (0.15047436546885562, -0.9806781334753645), (0.7617574165547447, 0.4763568144395216), (0.9160725920362325, 0.9823994209157814), (0.7548032671745575, 0.812465693071418), (0.3246690244662118, -0.3065870163715265), (-0.560268464119319, 0.9221103278210057), (0.4626072017685068, 0.15317773756585118), (0.6545368464007859, 0.9506033905866516), (0.5098893892275631, 0.7830254619622639), (-0.9966341637112501, 0.6861534690011151), (-0.385961778663223, 0.09296319784275209), (-0.6439053469798957, 0.8752161182375295), (0.5684180435556756, -0.7391524569031711), (-0.5741612839751691, -0.7694325902986678), (0.8870349468029142, -0.0626416154235423), (-0.7931632997873719, -0.1847117421834228), (-0.4170616485992771, -0.4058127574957917), (-0.012491529971446091, -0.7246396095112151), (-0.7011675969925535, -0.28220086603407024), (-0.1863646808060755, -0.9348786412235945), (0.7010535378320495, -0.8136524427155922), (-0.62997318656867, -0.5673706697790437), (0.20571346964486614, 0.8494676030862429), (-0.9582906806422553, 0.13304666328969184), (-0.9017636413609094, -0.7840101289980281), (-0.06844213395069287, 0.4449811706731899), (-0.6396613633959511, 0.7560570032865803), (0.17821628814178592, -0.6343776437447937), (0.644101079419162, -0.10231890186410197), (-0.9538502098940163, -0.4005239063329318), (-0.45575562738990283, 0.6260262571336148), (-0.022746028755935566, 0.7273548892777673), (-0.5843548196344637, -0.8970791745762465), (-0.134489546848358, -0.23508372044200576), (-0.4426578957700551, -0.4140837629408358), (-0.8960196125459656, 0.9648402032597405), (-0.5559247435005124, -0.9627298282745853), (-0.9816649175072354, 0.23566904569481584), (0.37830338720570555, 0.9659962089961276), (0.07032093120792404, -0.8448450554291562), (-0.043415679985484124, 0.2982047225662403), (-0.7426172167006169, -0.9450540467855513), (-0.7833001490152167, 0.8587880458484556), (0.5753231270739165, -0.3550369532737825), (0.9847762265520275, 0.5601827115776938), (0.7547992831124333, 0.28584939475098947), (-0.8448259708273849, -0.35476136665194846), (0.879827174886626, 0.88500567042747), (0.00876529646860802, -0.30836856894337306), (0.7747669072600987, 0.03129425244220663), (-0.9181318043271416, -0.1982212204694076), (-0.06470118131158964, 0.6372280272430135), (0.6586665563523673, -0.5780624300151294), (0.19158793464893065, -0.8718738170981424), (-0.6405141431119539, -0.5399480154588656), (-0.014859791537689349, 0.5613188554256261), (-0.5781514716971152, 0.2559171658208903), (-0.09436027264047842, -0.6637963328398158), (-0.6217035983366301, -0.4498016587418536), (-0.02539730686860331, -0.8160511451622021), (-0.1819750814557659, -0.48686881610529165), (-0.8311143734208255, 0.7345232437803397), (0.35720487022736136, 0.446537629591925), (0.040582999596036506, 0.27151188053807496), (-0.8582374284322447, -0.37397514967056456), (-0.2234253044314234, -0.5021431610668297), (0.061836579223409016, 0.2334741235096851), (-0.80899619273474, 0.061122116008547556), (0.09888582378400912, 0.13157005798444055), (0.4445239732731163, -0.00797318519075918), (-0.9232548892900476, 0.015610893892384903), (0.17258779115188272, -0.25500953936981885), (0.21398456462176796, 0.6500598840741447), (-0.3659869812873422, 0.29484229817933216), (-0.2749147404367873, -0.48538050541310906), (-0.5870409692233305, 0.282757960020392), (-0.009340854254250353, -0.5661727070520584), (0.715684675641828, 0.675955746635732), (-0.8905081791246903, 0.7234796547395257), (0.6429947842459898, 0.67193501065015), (0.42730640534452435, -0.925032998389832), (0.7829803203729666, 0.8067771780949617), (-0.9775718658097963, 0.5710827388382336), (0.0038854322067862768, 0.9691541535883614), (-0.7354493698452671, -0.7740512298723263), (0.6066203299748345, 0.7907036360462232), (-0.2224438094868928, -0.5466373525288557), (-0.6613459909460426, 0.17482481697589058), (-0.14091009854697045, -0.25741985123563094), (-0.6049343628632007, 0.7132474164128548), (0.6767791588126548, 0.46222825969908365), (-0.019881898612260862, 0.42921013485612036), (-0.024289173227493688, -0.6326047825520571), (0.9273609755463252, 0.9650020725623369), (0.43880148036486677, 0.9648468957926457), (0.7820211869090097, 0.7143859917652413), (0.10713771400332006, 0.87644702975162), (0.3603460364845872, 0.9734868379754906), (0.4646237172759058, -0.6227490350204024), (0.8685337181342596, -0.9486561516837511), (-0.4290526472599958, -0.07105881127723457), (-0.43449703896808, -0.43932628835061993), (0.8619314767156487, 0.959378844974651), (0.2524793886536818, -0.5086760933393686), (-0.968926808165882, -0.48180443592302513), (-0.9344049892159925, -0.9719354066164145), (0.8417012429814341, -0.846162566469103), (0.605011478012363, 0.6877711581719606), (0.29583324991339044, 0.3897339588911992), (0.3063194062122856, -0.4969091145392086), (-0.07403093613837086, -0.06210765636399951), (-0.8870943179841004, 0.13186549816749604), (-0.13905452148322728, -0.14200541685720025), (0.9059296265527592, -0.8674413586878444), (-0.8134297959876413, 0.8602852560727625), (0.5574039666928619, 0.5935991790218782), (-0.05988992995905518, 0.8331438388387209), (-0.15690479005758462, -0.09787464921091593), (-0.3499683268803646, -0.07036877228129979), (-0.7348170188722913, 0.6489097570964577), (0.08030663152612805, -0.4062556913602422), (-0.3948116107741906, -0.8419118545265418), (-0.7080619975080553, -0.9532463704924592), (0.9525099193607636, 0.7770440519594752), (0.8356696335520295, 0.11139106868318094), (0.37389056109299457, -0.10176149244871913), (-0.5179332268046919, 0.906755738195657), (-0.5101578900967876, -0.9795688509682434), (0.15559279752437916, -0.6175284856819807), (-0.3563748416756034, -0.7785942522281912), (-0.19025374281021157, 0.6459693559382012), (0.8948354100256368, -0.7186710154824598), (0.44374924585090136, 0.5837918431600173), (-0.4672648256848859, -0.3637024717244548), (-0.14026461181970284, 0.40814454352232743), (-0.8978006636593574, -0.7145974937988178), (0.11539804649202101, 0.14383105313034195), (-0.6062938753740512, 0.7502189488775537), (-0.4391572025223398, 0.9752356885866993), (-0.38715366747667934, -0.7985315162371476), (0.1255993226373906, 0.34908952025487716), (0.4225405126612505, 0.09137785183737068), (-0.13177079487908627, 0.22058823667211525), (-0.28381252837930226, 0.7711425341318037), (0.12869352408940005, -0.00020271374035063516), (-0.1448983949760465, -0.9880872282380235), (-0.0465272602088036, 0.3825699981736381), (-0.9077389357297825, 0.9009893307505754), (0.8110194938755062, -0.9800467040477308), (-0.01036226648551164, -0.20791141520235312), (-0.8491795910760915, -0.5161174572537066), (0.21510754077334227, 0.046435915914714965), (0.7015051124679705, -0.7139484315828417), (-0.8270226510406857, -0.037067448340805775), (0.3284454053998058, 0.7011277797513877), (0.8237931165115819, 0.7651378342874593), (0.3254212701250152, 0.9467579439573748), (0.6551253150336631, 0.09542055528534776), (0.059721730531613115, -0.2171872986083485), (0.4025247537451424, -0.23519298808428113), (-0.3656975388853696, 0.9488462674799432), (0.10234697465954268, -0.034042905991529704), (-0.021076834862308047, -0.30834904994034873), (-0.5425800701779875, -0.7901297323360688), (-0.5163096338241866, -0.0910731448043538), (0.775907526479902, -0.029927810692431223), (-0.5787010634781413, -0.17724734524515395), (-0.686139712926781, -0.6582853023191224), (-0.8201124220854916, 0.5324206558841762), (-0.5127613921846674, -0.46650367860717123), (0.4891824637526496, 0.061346961080255724), (-0.5571482675377373, 0.9411510539379417), (0.8707373419055771, 0.6207284256515975), (0.8219858954429966, -0.8779285711624081), (0.45726631268327833, 0.03270982726025018), (0.1853046782842407, -0.22408852515290034), (0.4255401852644969, -0.4449354754646797), (0.4317054766054196, -0.4691969281855446), (-0.5141023626569792, 0.7586320148155574), (0.7059121498036909, -0.2683408346536418), (0.09009019999661194, 0.7579800601346505), (0.747078544161313, -0.26381189782764225), (0.5335162579135664, -0.8201395593255578), (0.4399092773156905, 0.4001922617899951), (-0.2749366699470286, 0.22086561231547597), (0.09771281478875804, -0.8128675949424682), (0.8172959884714706, -0.22236236268899234), (-0.17248615077637086, 0.7974885426480312), (0.842252396378558, 0.7288571906820647), (0.7334957992325437, 0.07938913059765351), (0.39793866023322955, 0.8688245906412282), (0.7870291797356017, 0.06165030599614352), (-0.21473486896046423, 0.4392857648561843), (-0.5073085444740155, 0.4678281268067612), (-0.03340450051571797, 0.3363381514347139), (-0.30550362154772936, 0.7072959211525238), (-0.08235208900128632, 0.45302815156489107), (0.14964118143127214, -0.7335083814629508), (0.6494909995782758, -0.7605196537643031), (0.37417634836018854, -0.2745582477672208), (0.24402671028084932, -0.9659474167888), (0.6850340610798742, 0.3156075945350234), (-0.3353094019784082, 0.15936722855440655), (-0.35383706533582737, -0.4898161541205872), (0.08096129676569741, 0.984231777908692), (-0.09225205063861064, 0.11924130015978296), (-0.19117673582023098, -0.4570527844157015), (0.04484730422003991, -0.20431919075758143), (-0.214123335641055, -0.9923780601705212), (-0.5994170582611429, 0.6971508827916248), (0.8125161673575663, -0.5904108880645373), (-0.34462253292283207, 0.39677814793807964), (-0.4666775475043421, 0.7563498846928809), (-0.5353939698802537, 0.9372234589613415), (0.816330849123519, 0.7980310088153351), (-0.2442766721743026, 0.18660398148145663), (-0.795812123557933, -0.8439443207812483), (-0.7316221860345711, -0.00679059803926263), (0.029322252210048916, -0.039863229769652175), (0.8415655075406308, 0.2438708303677739), (-0.9738347839788026, 0.1820873912774983), (0.5812432844185678, -0.41807701493178806), (-0.11121009276117788, 0.09108995663689967), (0.4947780413182141, 0.731341348626044), (0.7549316153846815, 0.9940007248455749), (0.22703057664382742, -0.5973739903480808), (-0.5443322898224618, -0.9174453695604308), (0.32356085335906437, -0.4888945103497353), (0.359539113819318, -0.7408168135541069), (-0.22338680684947865, 0.28696110757103854), (-0.6034832533179653, -0.17871677245046125), (-0.1922451022511551, 0.6350068107028115), (-0.26205534723658164, -0.9428261933544171), (0.7931239883719934, 0.32715467498864514), (-0.37506617261760544, -0.1408871705323862), (0.060770083585807155, 0.8060976545229288), (-0.7736828407722232, -0.38511545538424485), (-0.4145664314854487, 0.5137234572891332), (0.05234994257136738, 0.832142969883906), (0.6007597711309947, 0.2405601651487772), (0.27450568752981797, -0.4408499884659134), (0.3389752585537307, 0.9808145570890736), (-0.5241924228584112, 0.2669518809982916), (-0.8763341953698638, -0.21111615960052754), (0.33981530567566987, -0.09232472231112476), (-0.8508778459163162, -0.2587109005842909), (0.8737211808432022, -0.5648439845189517), (0.8083427960826393, 0.11234062682554247), (-0.2712476783355593, -0.8774439655390605)] {
            assert_eq!(mandel(x, y, limit), f(x, y, limit));
        }
    }


    #[test]
    fn reg_fib() {
        let mut vm = reg::Vm::new();
        test_fib(|n| {
            vm.run(reg::FIB, &[n])
        });
    }

    #[test]
    fn reg_mandel() {
        let mut vm = reg::Vm::new();
        test_mandel(|x, y, n| {
            vm.run(reg::MANDEL, &[x, y, n])
        });
    }

    #[test]
    fn stack_fib_smart() {
        let mut vm = stack::Vm::new();
        test_fib(|n| {
            vm.run(stack::FIB_SMART, &[n])
        });
    }

    #[test]
    fn stack_fib_naive() {
        let mut vm = stack::Vm::new();
        test_fib(|n| {
            vm.run(stack::FIB_NAIVE, &[n])
        });
    }

    #[test]
    fn stack_mandel_smart() {
        let mut vm = stack::Vm::new();
        test_mandel(|x, y, n| {
            vm.run(stack::MANDEL_SMART, &[x, y, n])
        });
    }

    #[test]
    fn stack_mandel_naive() {
        let mut vm = stack::Vm::new();
        test_mandel(|x, y, n| {
            vm.run(stack::MANDEL_NAIVE, &[x, y, n])
        });
    }

    #[test]
    fn stack_mandel_smart_nops_slow() {
        let mut vm = stack::Vm::new();
        test_mandel(|x, y, n| {
            vm.run(stack::MANDEL_SMART_NOPS_SLOW, &[x, y, n])
        });
    }

    #[test]
    fn stack_mandel_smart_nops_same() {
        let mut vm = stack::Vm::new();
        test_mandel(|x, y, n| {
            vm.run(stack::MANDEL_SMART_NOPS_SAME, &[x, y, n])
        });
    }

    #[test]
    fn stack_mandel_smart_no_dup() {
        let mut vm = stack::Vm::new();
        test_mandel(|x, y, n| {
            vm.run(stack::MANDEL_SMART_NO_DUP, &[x, y, n])
        });
    }

    #[test]
    fn reg_add_chain() {
        let add_regs: [f64; 16] = core::array::from_fn(|i| i as f64);
        let mut vm = reg::Vm::new();
        assert_eq!(vm.run(&reg::ADD_CHAIN, &add_regs), (16*15/2) as f64);
    }

    #[test]
    fn reg_add_fast() {
        let add_regs: [f64; 16] = core::array::from_fn(|i| i as f64);
        let mut vm = reg::Vm::new();
        assert_eq!(vm.run(&reg::ADD_PAIRS, &add_regs), (16*15/2) as f64);
    }
}


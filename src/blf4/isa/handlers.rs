use crate::{
    blf4::{ByteRegister as Br, HandlerContext, TrapMode, WideRegister as Wr, R0},
    U4,
};

#[inline]
pub fn arg_pair<T, U, F1: FnOnce(U4) -> T, F2: FnOnce(U4) -> U>(
    c: &mut HandlerContext,
    f1: F1,
    f2: F2,
) -> OpRes<(T, U)> {
    let operand = c.fetch()?;
    let (a, b) = U4::paired(operand);
    Ok((f1(a), f2(b)))
}

#[inline]
pub fn arg_imm_byte(c: &mut HandlerContext) -> OpRes<u8> {
    let val = c.fetch()?;
    Ok(val)
}
#[inline]
pub fn arg_imm_wide(c: &mut HandlerContext) -> OpRes<u16> {
    let l = c.fetch()?;
    let h = c.fetch()?;
    Ok(u16::from_le_bytes([l, h]))
}

pub type OpRes<T = ()> = Result<T, TrapMode>;
pub type OpHandler = fn(c: &mut HandlerContext) -> OpRes;

pub static OP_HANDLERS: [OpHandler; 256] = {
    let mut handlers: [OpHandler; 256] = [n; 256];

    use super::*;

    handlers[NULL as usize] = n;
    handlers[HALT as usize] = halt;
    handlers[CTF as usize] = ctf;
    handlers[SYSCALL as usize] = syscall;
    handlers[RETH as usize] = reth;

    handlers[USR as usize] = usr;
    handlers[VMON as usize] = vmon;
    handlers[VMOFF as usize] = vmoff;
    handlers[PSTORE as usize] = pstore;
    handlers[PLOAD as usize] = pload;

    handlers[NOP as usize] = nop;
    handlers[PUSH_B as usize] = push_b;
    handlers[PUSH_W as usize] = push_w;
    handlers[POP_B as usize] = pop_b;
    handlers[POP_W as usize] = pop_w;
    handlers[CALL as usize] = call;
    handlers[RET as usize] = ret;
    handlers[STORE_BI as usize] = store_bi;
    handlers[STORE_WI as usize] = store_wi;
    handlers[STORE_BR as usize] = store_br;
    handlers[STORE_WR as usize] = store_wr;
    handlers[LOAD_BI as usize] = load_bi;
    handlers[LOAD_WI as usize] = load_wi;
    handlers[LOAD_BR as usize] = load_br;
    handlers[LOAD_WR as usize] = load_wr;
    handlers[JEZ as usize] = jez;
    handlers[JLT as usize] = jlt;
    handlers[JLE as usize] = jle;
    handlers[JGT as usize] = jgt;
    handlers[JGE as usize] = jge;
    handlers[JNZ as usize] = jnz;
    handlers[JO as usize] = jo;
    handlers[JNO as usize] = jno;
    handlers[JB as usize] = jb;
    handlers[JAE as usize] = jae;
    handlers[JA as usize] = ja;
    handlers[JBE as usize] = jbe;

    handlers[LDI_B as usize] = ldi_b;
    handlers[LDI_W as usize] = ldi_w;

    handlers[ADD_B as usize] = add_b;
    handlers[ADD_W as usize] = add_w;
    handlers[SUB_B as usize] = sub_b;
    handlers[SUB_W as usize] = sub_w;
    handlers[AND_B as usize] = and_b;
    handlers[AND_W as usize] = and_w;
    handlers[OR_B as usize] = or_b;
    handlers[OR_W as usize] = or_w;
    handlers[XOR_B as usize] = xor_b;
    handlers[XOR_W as usize] = xor_w;
    handlers[SHL_B as usize] = shl_b;
    handlers[SHL_W as usize] = shl_w;
    handlers[ASR_B as usize] = asr_b;
    handlers[ASR_W as usize] = asr_w;
    handlers[LSR_B as usize] = lsr_b;
    handlers[LSR_W as usize] = lsr_w;

    handlers[DIV_B as usize] = div_b;
    handlers[DIV_W as usize] = div_w;
    handlers[MUL_B as usize] = mul_b;
    handlers[MUL_W as usize] = mul_w;

    handlers
};

fn n(_c: &mut HandlerContext) -> OpRes {
    Err(TrapMode::Invalid)
}
fn halt(_c: &mut HandlerContext) -> OpRes {
    Err(TrapMode::Halt)
}
fn syscall(_c: &mut HandlerContext) -> OpRes {
    Err(TrapMode::SysCall)
}

fn ctf(c: &mut HandlerContext) -> OpRes {
    c.cpu.flags.trap = false;
    Ok(())
}
fn reth(c: &mut HandlerContext) -> OpRes {
    if !c.cpu.flags.trap {
        return Err(TrapMode::IllegalHandlerReturn);
    }
    c.pop_registers()?;
    c.cpu.flags.trap = false;
    Ok(())
}
fn usr(c: &mut HandlerContext) -> OpRes {
    if c.cpu.flags.user_mode {
        return Err(TrapMode::IllegalOperation);
    }
    c.cpu.flags.user_mode = true;

    Ok(())
}
fn vmon(c: &mut HandlerContext) -> OpRes {
    if c.cpu.flags.user_mode {
        return Err(TrapMode::IllegalOperation);
    }
    c.cpu.flags.virtual_mode = true;

    Ok(())
}
fn vmoff(c: &mut HandlerContext) -> OpRes {
    if c.cpu.flags.user_mode {
        return Err(TrapMode::IllegalOperation);
    }
    c.cpu.flags.virtual_mode = false;

    Ok(())
}
fn pstore(c: &mut HandlerContext) -> OpRes {
    if c.cpu.flags.user_mode {
        return Err(TrapMode::IllegalOperation);
    }
    let (br1, wr) = arg_pair(c, Br, Wr)?;
    let (br2, zero) = arg_pair(c, Br, u8::from)?;

    if zero != 0 {
        return Err(TrapMode::Invalid);
    }

    let high_byte = c.cpu.read_br(br1);
    let low_wide = c.cpu.read_wr(wr)?;
    let addr = low_wide as u32 | ((high_byte as u32) << 16);

    c.physical_write(addr, c.cpu.read_br(br2))
}
fn pload(c: &mut HandlerContext) -> OpRes {
    if c.cpu.flags.user_mode {
        return Err(TrapMode::IllegalOperation);
    }
    let (br1, br2) = arg_pair(c, Br, Br)?;
    let (wr, zero) = arg_pair(c, Wr, u8::from)?;

    if zero != 0 {
        return Err(TrapMode::Invalid);
    }

    let high_byte = c.cpu.read_br(br2);
    let low_wide = c.cpu.read_wr(wr)?;
    let addr = low_wide as u32 | ((high_byte as u32) << 16);

    let val = c.physical_read(addr)?;
    c.cpu.write_br(br1, val);

    Ok(())
}

#[inline]
fn binop_b(
    c: &mut HandlerContext,
    binop: fn(u8, u8) -> (u8, bool),
    ibinop: fn(i8, i8) -> (i8, bool),
) -> OpRes {
    let (r1, r2) = arg_pair(c, Br, Br)?;
    let (r3, r4) = arg_pair(c, Br, u8::from)?;

    let r2 = c.cpu.read_br(r2);
    let r3 = c.cpu.read_br(r3);
    if r4 != 0 {
        return Err(TrapMode::Invalid);
    }

    let (res, carry) = binop(r2, r3);
    let (ires, overflowing) = ibinop(r2 as i8, r3 as i8);
    c.cpu.flags.carry = carry;
    c.cpu.flags.overflow = overflowing;
    c.cpu.flags.sign = ires.is_negative();
    c.cpu.flags.zero = res == 0;

    c.cpu.write_br(r1, res);

    Ok(())
}
#[inline]
fn binop_w(
    c: &mut HandlerContext,
    binop: fn(u16, u16) -> (u16, bool),
    ibinop: fn(i16, i16) -> (i16, bool),
) -> OpRes {
    let (r1, r2) = arg_pair(c, Wr, Wr)?;
    let (r3, r4) = arg_pair(c, Wr, u8::from)?;

    let r2 = c.cpu.read_wr(r2)?;
    let r3 = c.cpu.read_wr(r3)?;
    if r4 != 0 {
        return Err(TrapMode::Invalid);
    }

    let (res, carry) = binop(r2, r3);
    let (ires, overflowing) = ibinop(r2 as i16, r3 as i16);
    c.cpu.flags.carry = carry;
    c.cpu.flags.overflow = overflowing;
    c.cpu.flags.sign = ires.is_negative();
    c.cpu.flags.zero = res == 0;

    c.cpu.write_wr(r1, res)?;

    Ok(())
}

fn add_b(c: &mut HandlerContext) -> OpRes {
    binop_b(c, u8::overflowing_add, i8::overflowing_add)
}
fn add_w(c: &mut HandlerContext) -> OpRes {
    binop_w(c, u16::overflowing_add, i16::overflowing_add)
}
fn sub_b(c: &mut HandlerContext) -> OpRes {
    binop_b(c, u8::overflowing_sub, i8::overflowing_sub)
}
fn sub_w(c: &mut HandlerContext) -> OpRes {
    binop_w(c, u16::overflowing_sub, i16::overflowing_sub)
}
fn and_b(c: &mut HandlerContext) -> OpRes {
    binop_b(c, |x, y| (x & y, false), |x, y| (x & y, false))
}
fn and_w(c: &mut HandlerContext) -> OpRes {
    binop_w(c, |x, y| (x & y, false), |x, y| (x & y, false))
}
fn or_b(c: &mut HandlerContext) -> OpRes {
    binop_b(c, |x, y| (x | y, false), |x, y| (x | y, false))
}
fn or_w(c: &mut HandlerContext) -> OpRes {
    binop_w(c, |x, y| (x | y, false), |x, y| (x | y, false))
}
fn xor_b(c: &mut HandlerContext) -> OpRes {
    binop_b(c, |x, y| (x ^ y, false), |x, y| (x ^ y, false))
}
fn xor_w(c: &mut HandlerContext) -> OpRes {
    binop_w(c, |x, y| (x ^ y, false), |x, y| (x ^ y, false))
}
fn shl_b(c: &mut HandlerContext) -> OpRes {
    binop_b(c, |x, y| (x << y, false), |x, y| (x << y, false))
}
fn shl_w(c: &mut HandlerContext) -> OpRes {
    binop_w(c, |x, y| (x << y, false), |x, y| (x << y, false))
}
fn asr_b(c: &mut HandlerContext) -> OpRes {
    binop_b(
        c,
        |x, y| (((x as i8) >> y) as u8, false),
        |x, y| (x >> y, false),
    )
}
fn asr_w(c: &mut HandlerContext) -> OpRes {
    binop_w(
        c,
        |x, y| (((x as i16) >> y) as u16, false),
        |x, y| (x >> y, false),
    )
}
fn lsr_b(c: &mut HandlerContext) -> OpRes {
    binop_b(
        c,
        |x, y| (x >> y, false),
        |x, y| (((x as u8) >> y) as i8, false),
    )
}
fn lsr_w(c: &mut HandlerContext) -> OpRes {
    binop_w(
        c,
        |x, y| (x >> y, false),
        |x, y| (((x as u16) >> y) as i16, false),
    )
}
fn mul_b(c: &mut HandlerContext) -> OpRes {
    let (r1, r2) = arg_pair(c, Br, Br)?;
    let (r3, r4) = arg_pair(c, Br, Br)?;

    let res = c.cpu.read_br(r3) as u16 * c.cpu.read_br(r4) as u16;
    let [lower, upper] = res.to_le_bytes();

    c.cpu.flags.carry = upper != 0;
    c.cpu.flags.overflow = c.cpu.flags.carry;
    c.cpu.flags.zero = lower == 0;
    c.cpu.flags.sign = (lower as i8).is_negative();

    c.cpu.write_br(r1, upper);
    c.cpu.write_br(r2, lower);

    Ok(())
}
fn mul_w(c: &mut HandlerContext) -> OpRes {
    let (r1, r2) = arg_pair(c, Wr, Wr)?;
    let (r3, r4) = arg_pair(c, Wr, Wr)?;

    let res = c.cpu.read_wr(r3)? as u32 * c.cpu.read_wr(r4)? as u32;
    let [lower1, lower2, upper1, upper2] = res.to_le_bytes();
    let lower = u16::from_le_bytes([lower1, lower2]);
    let upper = u16::from_le_bytes([upper1, upper2]);

    c.cpu.flags.carry = upper != 0;
    c.cpu.flags.overflow = c.cpu.flags.carry;
    c.cpu.flags.zero = lower == 0;
    c.cpu.flags.sign = (lower as i16).is_negative();

    c.cpu.write_wr(r1, upper)?;
    c.cpu.write_wr(r2, lower)?;

    Ok(())
}
fn div_b(c: &mut HandlerContext) -> OpRes {
    let (r1, r2) = arg_pair(c, Br, Br)?;
    let (r3, r4) = arg_pair(c, Br, Br)?;

    let n1 = c.cpu.read_br(r3);
    let n2 = c.cpu.read_br(r4);
    if n2 == 0 {
        return Err(TrapMode::ZeroDiv);
    }
    let upper = n1 / n2;
    let lower = n1 % n2;

    c.cpu.write_br(r1, upper);
    c.cpu.write_br(r2, lower);

    Ok(())
}
fn div_w(c: &mut HandlerContext) -> OpRes {
    let (r1, r2) = arg_pair(c, Wr, Wr)?;
    let (r3, r4) = arg_pair(c, Wr, Wr)?;

    let n1 = c.cpu.read_wr(r3)?;
    let n2 = c.cpu.read_wr(r4)?;
    if n2 == 0 {
        return Err(TrapMode::ZeroDiv);
    }
    let upper = n1 / n2;
    let lower = n1 % n2;

    c.cpu.write_wr(r1, upper)?;
    c.cpu.write_wr(r2, lower)?;

    Ok(())
}

fn nop(_c: &mut HandlerContext) -> OpRes {
    Ok(())
}
fn push_b(c: &mut HandlerContext) -> OpRes {
    let (b, z) = arg_pair(c, Br, u8::from)?;
    let b = c.cpu.read_br(b);
    if z != 0 {
        return Err(TrapMode::Invalid);
    }
    c.pushb(b)?;

    Ok(())
}
fn push_w(c: &mut HandlerContext) -> OpRes {
    let (w, z) = arg_pair(c, Wr, u8::from)?;
    let w = c.cpu.read_wr(w)?;
    if z != 0 {
        return Err(TrapMode::Invalid);
    }
    c.pushw(w)?;

    Ok(())
}
fn pop_b(c: &mut HandlerContext) -> OpRes {
    let (r1, z) = arg_pair(c, Br, u8::from)?;
    if z != 0 {
        return Err(TrapMode::Invalid);
    }

    let b = c.popb()?;
    c.cpu.write_br(r1, b);

    Ok(())
}
fn pop_w(c: &mut HandlerContext) -> OpRes {
    let (r1, z) = arg_pair(c, Wr, u8::from)?;
    if z != 0 {
        return Err(TrapMode::Invalid);
    }

    let w = c.popw()?;
    c.cpu.write_wr(r1, w)?;

    Ok(())
}
fn call(c: &mut HandlerContext) -> OpRes {
    let w = arg_imm_wide(c)?;
    c.cpu.link = c.cpu.program_counter;
    c.cpu.program_counter = w;

    Ok(())
}
fn ret(c: &mut HandlerContext) -> OpRes {
    let b = arg_imm_byte(c)?;
    c.cpu.stack += b as u16;
    c.cpu.program_counter = c.cpu.link;

    Ok(())
}
fn store_bi(c: &mut HandlerContext) -> OpRes {
    let (r1, r2) = arg_pair(c, Wr, Br)?;
    let offset = arg_imm_wide(c)?;

    let addr = c.cpu.read_wr(r1)? + offset;
    c.write(addr, c.cpu.read_br(r2))?;

    Ok(())
}
fn store_br(c: &mut HandlerContext) -> OpRes {
    let (r1, r2) = arg_pair(c, Wr, Wr)?;
    let (r3, z) = arg_pair(c, Br, u8::from)?;
    if z != 0 {
        return Err(TrapMode::Invalid);
    }
    let offset = c.cpu.read_wr(r2)?;

    let addr = c.cpu.read_wr(r1)? + offset;
    c.write(addr, c.cpu.read_br(r3))?;

    Ok(())
}
fn store_wi(c: &mut HandlerContext) -> OpRes {
    let (r1, r2) = arg_pair(c, Wr, Wr)?;
    let offset = arg_imm_wide(c)?;

    let addr = c.cpu.read_wr(r1)? + offset;
    c.write_wide(addr, c.cpu.read_wr(r2)?)?;

    Ok(())
}
fn store_wr(c: &mut HandlerContext) -> OpRes {
    let (r1, r2) = arg_pair(c, Wr, Wr)?;
    let (r3, z) = arg_pair(c, Wr, u8::from)?;
    if z != 0 {
        return Err(TrapMode::Invalid);
    }
    let offset = c.cpu.read_wr(r2)?;

    let addr = c.cpu.read_wr(r1)? + offset;
    c.write_wide(addr, c.cpu.read_wr(r3)?)?;

    Ok(())
}
fn load_bi(c: &mut HandlerContext) -> OpRes {
    let (r1, r2) = arg_pair(c, Br, Wr)?;
    let offset = arg_imm_wide(c)?;

    let addr = c.cpu.read_wr(r2)? + offset;
    let val = c.read(addr)?;
    c.cpu.write_br(r1, val);

    Ok(())
}
fn load_br(c: &mut HandlerContext) -> OpRes {
    let (r1, r2) = arg_pair(c, Br, Wr)?;
    let (r3, z) = arg_pair(c, Wr, u8::from)?;
    if z != 0 {
        return Err(TrapMode::Invalid);
    }
    let offset = c.cpu.read_wr(r3)?;

    let addr = c.cpu.read_wr(r2)? + offset;
    let val = c.read(addr)?;
    c.cpu.write_br(r1, val);

    Ok(())
}
fn load_wi(c: &mut HandlerContext) -> OpRes {
    let (r1, r2) = arg_pair(c, Wr, Wr)?;
    let offset = arg_imm_wide(c)?;

    let addr = c.cpu.read_wr(r2)? + offset;
    let val = c.read_wide(addr)?;
    c.cpu.write_wr(r1, val)?;

    Ok(())
}
fn load_wr(c: &mut HandlerContext) -> OpRes {
    let (r1, r2) = arg_pair(c, Wr, Wr)?;
    let (r3, z) = arg_pair(c, Wr, u8::from)?;
    if z != 0 {
        return Err(TrapMode::Invalid);
    }
    let offset = c.cpu.read_wr(r3)?;

    let addr = c.cpu.read_wr(r2)? + offset;
    let val = c.read_wide(addr)?;
    c.cpu.write_wr(r1, val)?;

    Ok(())
}

fn jez(c: &mut HandlerContext) -> OpRes {
    jif(c.cpu.flags.zero, c)
}
fn jlt(c: &mut HandlerContext) -> OpRes {
    jif(c.cpu.flags.sign != c.cpu.flags.overflow, c)
}
fn jle(c: &mut HandlerContext) -> OpRes {
    jif(
        c.cpu.flags.sign != c.cpu.flags.overflow && c.cpu.flags.zero,
        c,
    )
}
fn jgt(c: &mut HandlerContext) -> OpRes {
    jif(
        c.cpu.flags.sign == c.cpu.flags.overflow && !c.cpu.flags.zero,
        c,
    )
}
fn jge(c: &mut HandlerContext) -> OpRes {
    jif(c.cpu.flags.sign == c.cpu.flags.overflow, c)
}
fn jnz(c: &mut HandlerContext) -> OpRes {
    jif(!c.cpu.flags.zero, c)
}
fn jo(c: &mut HandlerContext) -> OpRes {
    jif(c.cpu.flags.overflow, c)
}
fn jno(c: &mut HandlerContext) -> OpRes {
    jif(!c.cpu.flags.overflow, c)
}
fn ja(c: &mut HandlerContext) -> OpRes {
    jif(!c.cpu.flags.carry && !c.cpu.flags.zero, c)
}
fn jae(c: &mut HandlerContext) -> OpRes {
    jif(!c.cpu.flags.carry, c)
}
fn jb(c: &mut HandlerContext) -> OpRes {
    jif(c.cpu.flags.carry, c)
}
fn jbe(c: &mut HandlerContext) -> OpRes {
    jif(c.cpu.flags.carry || c.cpu.flags.zero, c)
}
fn jif(cond: bool, c: &mut HandlerContext) -> OpRes {
    let location = arg_imm_wide(c)?;
    if cond {
        c.cpu.program_counter = location;
    }
    Ok(())
}

fn ldi_b(c: &mut HandlerContext) -> OpRes {
    let (r1, z) = arg_pair(c, Br, u8::from)?;
    if z != 0 {
        return Err(TrapMode::Invalid);
    }

    let b = arg_imm_byte(c)?;

    c.cpu.write_br(r1, b);

    Ok(())
}
fn ldi_w(c: &mut HandlerContext) -> OpRes {
    let (r1, o) = arg_pair(c, Wr, u8::from)?;

    let w = arg_imm_wide(c)?;

    match o {
        // ldi
        0 => c.cpu.write_wr(r1, w)?,
        // jmp, jump
        1 => {
            if r1 == R0 {
                // jmp imm
                c.cpu.program_counter = w;
            } else {
                // jmp c
                c.cpu.program_counter = c.cpu.read_wr(r1)?;
            }
        }
        _ => return Err(TrapMode::Invalid),
    }

    Ok(())
}

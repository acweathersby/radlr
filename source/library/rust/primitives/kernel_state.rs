use crate::bytecode::constants::NORMAL_STATE_MASK;

use super::KernelStack;
use super::KernelToken;

pub struct KernelState
{
    pub stack:           KernelStack,
    pub tokens:          [KernelToken; 3],
    pub active_state:    u32,
    pub sym_accumulator: u32,
    pub production_id:   u32,
    pub pointer:         u32,
    pub in_peek_mode:    bool,
    pub in_fail_mode:    bool,
    pub is_scanner:      bool,
    pub interrupted:     bool,
}

impl KernelState
{
    pub fn new() -> Self
    {
        Self {
            tokens:          [
                KernelToken::new(),
                KernelToken::new(),
                KernelToken::new(),
            ],
            stack:           KernelStack::new(),
            active_state:    0,
            sym_accumulator: 0,
            production_id:   0,
            pointer:         0,
            in_fail_mode:    false,
            in_peek_mode:    false,
            is_scanner:      false,
            interrupted:     false,
        }
    }

    #[inline]
    pub fn get_active_state(&mut self) -> u32
    {
        self.active_state
    }

    #[inline]
    pub fn set_active_state(&mut self, active_state: u32)
    {
        self.active_state = active_state
    }

    #[inline]
    pub fn set_anchor_token(&mut self, token: KernelToken)
    {
        self.tokens[0] = token;
    }

    #[inline]
    pub fn set_assert_token(&mut self, token: KernelToken)
    {
        self.tokens[1] = token;
    }

    #[inline]
    pub fn set_peek_token(&mut self, token: KernelToken)
    {
        self.tokens[2] = token;
    }

    #[inline]
    pub fn get_anchor_token(&mut self) -> KernelToken
    {
        self.tokens[0]
    }

    #[inline]
    pub fn get_assert_token(&mut self) -> KernelToken
    {
        self.tokens[1]
    }

    #[inline]
    pub fn get_peek_token(&mut self) -> KernelToken
    {
        self.tokens[2]
    }

    #[inline]
    pub fn init_normal_state(&mut self, state_offset: u32)
    {
        self.stack.reset(NORMAL_STATE_MASK | state_offset);
    }

    #[inline]
    pub fn set_production(&mut self, production_id: u32)
    {
        self.production_id = production_id;
    }

    #[inline]
    pub fn get_production(&mut self) -> u32
    {
        self.production_id
    }

    #[inline]
    pub fn is_scanner(&self) -> bool
    {
        self.is_scanner
    }

    #[inline]
    pub fn make_scanner(&mut self)
    {
        self.is_scanner = true
    }
}

#![allow(unused)]
use std::collections::HashMap;

// Define a struct to hold register allocation information, including spill
// location
#[derive(Debug, Clone)]
pub struct AllocationResult<'a> {
  pub result_register:         &'a str,
  pub dividend_register:       &'a str,
  pub divisor_register:        &'a str,
  pub new_free_registers_mask: u64,
  pub spilled_register:        Option<(&'a str, usize)>, // (register name, memory location)
}

// Define a simple spill function (in a real scenario, this would be more
// complex)
fn spill_register(free_registers_mask: u64, register_map: &HashMap<u8, &'static str>) -> Option<(&'static str, u64)> {
  // Find the first non-reserved free register to spill (simplistic strategy)
  for i in 0..64 {
    if (free_registers_mask >> i) & 1 != 0 {
      if let Some(reg) = register_map.get(&(i as u8)) {
        // Basic exclusion of stack and base pointers for spilling in this example
        if *reg != "rsp" && *reg != "rbp" {
          return Some((*reg, i as u64 * 8)); // Example memory location
        }
      }
    }
  }
  None // No suitable register to spill
}

pub fn allocate_registers_for_division_with_spill(
  dividend_index: usize,
  divisor_index: usize,
  free_registers_mask: u64,
  spilled_registers: &mut HashMap<&'static str, usize>, // Track spilled registers and their locations
) -> Result<AllocationResult<'static>, &'static str> {
  let register_map: HashMap<u8, &'static str> = [
    (0, "rax"),
    (1, "rcx"),
    (2, "rdx"),
    (3, "rbx"),
    (4, "rsi"),
    (5, "rdi"),
    (6, "rsp"),
    (7, "rbp"),
    (8, "r8"),
    (9, "r9"),
    (10, "r10"),
    (11, "r11"),
    (12, "r12"),
    (13, "r13"),
    (14, "r14"),
    (15, "r15"),
  ]
  .iter()
  .cloned()
  .collect();

  let reverse_register_map: HashMap<&'static str, u8> = register_map.iter().map(|(key, val)| (*val, *key)).collect();

  let mut available_registers = Vec::new();
  for i in 0..64 {
    if (free_registers_mask >> i) & 1 != 0 {
      if let Some(reg) = register_map.get(&(i as u8)) {
        available_registers.push(*reg);
      }
    }
  }

  if available_registers.len() < 2 {
    // Not enough registers, attempt to spill
    if let Some((spilled_reg, mem_location)) = spill_register(free_registers_mask, &register_map) {
      println!("Spilling register {} to memory location {}", spilled_reg, mem_location);
      spilled_registers.insert(spilled_reg, mem_location as usize);
      let spilled_reg_bit = reverse_register_map.get(spilled_reg).unwrap();
      let new_free_mask_after_spill = free_registers_mask | (1 << spilled_reg_bit); // Mark as free for allocation
      dbg!(&spilled_registers);
      // Retry allocation after spill (recursive call might be cleaner in a real
      // allocator)
      return allocate_registers_for_division_with_spill(
        dividend_index,
        divisor_index,
        new_free_mask_after_spill,
        spilled_registers,
      );
    } else {
      return Err("Not enough registers available and no suitable register to spill.");
    }
  }

  let mut dividend_register: Option<&'static str> = None;
  let mut divisor_register: Option<&'static str> = None;
  let mut new_free_registers_mask = free_registers_mask;
  let mut spilled = None;

  // Prioritize "rax" for the dividend
  if let Some(&rax_bit) = reverse_register_map.get("rax") {
    if (free_registers_mask & (1 << rax_bit)) != 0 {
      dividend_register = Some("rax");
      new_free_registers_mask &= !(1 << rax_bit);

      for i in 0..64 {
        if (new_free_registers_mask >> i) & 1 != 0 {
          if let Some(reg) = register_map.get(&(i as u8)) {
            if *reg != "rax" {
              divisor_register = Some(*reg);
              new_free_registers_mask &= !(1 << i);
              break;
            }
          }
        }
      }
      if divisor_register.is_none() {
        // Need to spill to get a divisor register
        if let Some((spilled_reg, mem_location)) = spill_register(new_free_registers_mask, &register_map) {
          println!("Spilling register {} to memory location (divisor) {}", spilled_reg, mem_location);
          spilled_registers.insert(spilled_reg, mem_location as usize);
          let spilled_reg_bit = reverse_register_map.get(spilled_reg).unwrap();
          divisor_register = Some(spilled_reg);
          new_free_registers_mask &= !(1 << spilled_reg_bit);
          spilled = Some((spilled_reg, mem_location as usize));
        } else {
          return Err("Not enough registers available for the divisor and no suitable register to spill.");
        }
      }
    }
  }

  if dividend_register.is_none() || divisor_register.is_none() {
    // Allocate without rax priority, may need to spill
    let mut first_available: Option<&'static str> = None;
    let mut second_available: Option<&'static str> = None;
    let mut temp_mask = new_free_registers_mask; // Use potentially updated mask

    for i in 0..64 {
      if (temp_mask >> i) & 1 != 0 {
        if let Some(reg) = register_map.get(&(i as u8)) {
          if first_available.is_none() {
            first_available = Some(*reg);
            temp_mask &= !(1 << i);
          } else if second_available.is_none() {
            second_available = Some(*reg);
            break;
          }
        }
      }
    }

    if let Some(reg) = first_available {
      dividend_register = Some(reg);
      if let Some(&bit) = reverse_register_map.get(reg) {
        new_free_registers_mask &= !(1 << bit);
      }
    } else {
      // Spill for dividend
      if let Some((spilled_reg, mem_location)) = spill_register(new_free_registers_mask, &register_map) {
        println!("Spilling register {} to memory location (dividend) {}", spilled_reg, mem_location);
        spilled_registers.insert(spilled_reg, mem_location as usize);
        let spilled_reg_bit = reverse_register_map.get(spilled_reg).unwrap();
        dividend_register = Some(spilled_reg);
        new_free_registers_mask &= !(1 << spilled_reg_bit);
        spilled = Some((spilled_reg, mem_location as usize));
      } else {
        return Err("Not enough registers available for the dividend and no suitable register to spill.");
      }
    }

    if second_available.is_some() {
      divisor_register = second_available;
      if let Some(&bit) = reverse_register_map.get(divisor_register.unwrap()) {
        new_free_registers_mask &= !(1 << bit);
      }
    } else {
      // Spill for divisor
      if let Some((spilled_reg, mem_location)) = spill_register(new_free_registers_mask, &register_map) {
        println!("Spilling register {} to memory location (divisor) {}", spilled_reg, mem_location);
        spilled_registers.insert(spilled_reg, mem_location as usize);
        let spilled_reg_bit = reverse_register_map.get(spilled_reg).unwrap();
        divisor_register = Some(spilled_reg);
        new_free_registers_mask &= !(1 << spilled_reg_bit);
        if spilled.is_none() {
          spilled = Some((spilled_reg, mem_location as usize));
        }
      } else {
        return Err("Not enough registers available for the divisor and no suitable register to spill.");
      }
    }
  }

  Ok(AllocationResult {
    result_register: "rax",
    dividend_register: dividend_register.unwrap(),
    divisor_register: divisor_register.unwrap(),
    new_free_registers_mask,
    spilled_register: spilled,
  })
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::collections::HashMap;

  #[test]
  fn test_allocate_with_spill_needed() {
    let free_mask = 0b0000000000000001; // Only rax is free
    let mut spilled = HashMap::new();
    let result = allocate_registers_for_division_with_spill(1, 3, free_mask, &mut spilled);
    assert!(result.is_ok());
    let alloc_result = result.unwrap();
    assert_eq!(alloc_result.result_register, "rax");
    assert!(alloc_result.dividend_register != "rax");
    assert!(alloc_result.divisor_register != "rax" && alloc_result.divisor_register != alloc_result.dividend_register);
    assert!(alloc_result.spilled_register.is_some());
    assert_eq!(spilled.len(), 1);
  }

  #[test]
  fn test_allocate_without_spill() {
    let free_mask = 0b0000000000000111; // rax, rcx, rdx are free
    let mut spilled = HashMap::new();
    let result = allocate_registers_for_division_with_spill(1, 3, free_mask, &mut spilled);
    assert!(result.is_ok());
    let alloc_result = result.unwrap();
    assert_eq!(alloc_result.result_register, "rax");
    assert!(alloc_result.spilled_register.is_none());
    assert!(spilled.is_empty());
  }

  #[test]
  fn test_allocate_no_spill_possible() {
    let free_mask = 0b0000000000000000; // No registers free
    let mut spilled = HashMap::new();
    let result = allocate_registers_for_division_with_spill(1, 3, free_mask, &mut spilled);
    assert!(result.is_err());
    assert_eq!(result.err().unwrap(), "Not enough registers available and no suitable register to spill.");
    assert!(spilled.is_empty());
  }

  #[test]
  fn test_allocate_spill_for_divisor() {
    let free_mask = 0b0000000000000001; // Only rax is free
    let mut spilled = HashMap::new();
    let result = allocate_registers_for_division_with_spill(1, 3, free_mask, &mut spilled);
    assert!(result.is_ok());
    let alloc_result = result.unwrap();
    assert_eq!(alloc_result.result_register, "rax");
    assert!(alloc_result.dividend_register != "rax");
    assert!(alloc_result.divisor_register != "rax" && alloc_result.divisor_register != alloc_result.dividend_register);
    assert!(alloc_result.spilled_register.is_some());
    assert_eq!(spilled.len(), 1);
    // In this specific case, 'rax' is used for dividend, so another register
    // must be spilled for the divisor.
  }
}

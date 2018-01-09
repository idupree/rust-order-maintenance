
// reference info:
// Bender et al, Two Simplified Algorithms for Maintaining Order in a List
// https://www.ics.uci.edu/~eppstein/PADS/OrderedSequence.py
// https://www.ics.uci.edu/~eppstein/PADS/Sequence.py
// https://www.ics.uci.edu/~eppstein/PADS/ABOUT-PADS.txt
// (MIT license, if it even applied)

use std::cmp::Ordering;
use std::collections::HashMap;
use std::hash::Hash;
use std::cmp::Eq;
use std::iter::FromIterator;

use std::fmt::Debug;

type Tag = u64;

#[derive(Debug)]
struct Position<T> {
    prev: T,
    next: T,
    tag: Tag,
}

// sorry about the Clone, todo maybe index prev/next by tag somehow?
// also maybe TODO custom Eq that treats tag exact values as irrelevant?
// possibly by an iter that does something interesting
#[derive(Debug)]
pub struct OrderMaintenance<T>
    where T: Hash + Eq + Clone {
    positions: HashMap<T, Position<T>>,
    front: Option<T>,
}
#[derive(Debug)]
pub struct IterWithTag<'a, T>
    where T: Hash + Eq + Clone + 'a {
    om: &'a OrderMaintenance<T>,
    first: Option<T>,
    current: Option<T>,
}
impl<'a, T> Iterator for IterWithTag<'a, T>
    where T: Hash + Eq + Clone {
    type Item = (T, Tag);
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(current) = self.current.clone() {
            let current_position: &Position<T> = self.om.positions.get(&current).unwrap();
            let next = Some(current_position.next.clone());
            if next != self.first {
                self.current = next;
            } else {
                self.current = None;
            }
            Some((current, current_position.tag))
        } else {
            None
        }
    }
}


impl<T> OrderMaintenance<T>
    where T: Hash + Eq + Clone + Debug {
    pub fn new() -> OrderMaintenance<T> {
        OrderMaintenance { positions: HashMap::new(), front: None }
    }
    pub fn debug(&self) {
        eprintln!("om:{:?} full {:?}", Vec::from_iter(self.iter_values_with_tags()), self);
    }
    pub fn iter_values_with_tags<'a>(&'a self) -> IterWithTag<'a, T> {
        let front = self.front().map(|t| t.clone());
        IterWithTag{om: self, first: front.clone(), current: front.clone()}
    }
    pub fn compare(&self, a: &T, b: &T) -> Option<Ordering> {
        let a_tag = self.positions.get(a)?.tag;
        let b_tag = self.positions.get(b)?.tag;
        Some(a_tag.cmp(&b_tag))
    }
    pub fn remove(&mut self, value: &T) -> bool {
        if let Some(position) = self.positions.remove(value) {
            let prev = position.prev.clone();
            let next = position.next.clone();
            self.positions.get_mut(&position.prev).map(|p| { p.next = next; });
            self.positions.get_mut(&position.next).map(|p| { p.prev = prev; });
            true
        } else {
            false
        }
    }
    pub fn insert_only(&mut self, value: T) {
        assert!(self.len() == 0);
        self.positions.insert(value.clone(), Position {
            prev: value.clone(),
            next: value.clone(),
            tag: 0
        });
        self.front = Some(value.clone());
        self.debug();
    }
    pub fn insert_after(&mut self, after: &T, value: T) {
        // todo:
        // error if no after
        // error if value is already somewhere (else)
        // error if after == value
        let (prev_tag, next) = {
                let prev_position = self.positions.get(after).unwrap();
                (prev_position.tag, prev_position.next.clone())
            };
        let next_tag = self.positions.get(&next).unwrap().tag;
        // TODO: wrapping, mid way, etc ?
        let tag = if prev_tag == Tag::max_value() { prev_tag } else { prev_tag + 1 };
        let position = Position {
                prev: after.clone(),
                next: next.clone(),
                tag: tag,
            };
        self.positions.insert(value.clone(), position);
        self.positions.get_mut(after).map(|p| { p.next = value.clone() });
        self.positions.get_mut(&next).map(|p| { p.prev = value.clone() });
        if tag == prev_tag || tag == next_tag {
            self.rebalance(&value);
        }
        self.debug();
        self.verify_valid_structure();
    }
    pub fn len(&self) -> usize {
        self.positions.len()
    }
    fn front(&self) -> Option<T> {
        self.front.clone()
        /*if let Some((value1, position1)) = self.positions.iter().next() {
            let mut value: T = value1.clone();
            let mut prev: T = position1.prev.clone();
            let mut lowest_tag: Tag = position1.tag;
            let lowest_value = loop {
                if let Some(prev_position) = self.positions.get(&prev) {
                    if prev_position.tag < lowest_tag {
                        value = prev;
                        prev = prev_position.prev.clone();
                        lowest_tag = prev_position.tag;
                        continue;
                    }
                }
                break value.clone();
            };
            Some(lowest_value)
        } else {
            None
        }*/
    }
    /*
    fn next_cyclic<'a, 'b>(&'a self, value: &'b T) -> &'a T {
        &self.positions.get(value).unwrap().next
    }
    fn next_linear<'a, 'b>(&'a self, value: &'b T) -> Option<&'a T> {
        let next = self.next_cyclic(value);
        if next == value || self.positions.get(next).unwrap().tag < self.positions.get(value).unwrap().tag {
            None
        } else {
            Some(next)
        }
    }
    */
    fn verify_list_integrity(&self) {
        if let Some(ref front) = self.front {
            let mut value: &T = &front;
            let mut next: &T = &self.positions.get(&front).expect("front not in positions").next;
            let mut num_seen: u64 = 0;
            loop {
                num_seen += 1;
                if let Some(ref next_position) = self.positions.get(&next) {
                    if &next_position.prev != value {
                        panic!("integrity of prev/next");
                    }
                    if next == front {
                        break;
                    }
                    value = next;
                    next = &next_position.next;
                } else {
                    panic!("there should always be a next");
                }
            }
            if num_seen != self.positions.len() as u64 {
                panic!("not all seen");
            }
        } else {
            if self.positions.len() != 0 {
                panic!("positions but no front");
            }
        }
    }
    fn verify_valid_structure(&self) {
        self.verify_list_integrity();
        let mut previous_tag: Option<Tag> = None;
        let mut num_seen: u64 = 0;
        for (_, tag) in self.iter_values_with_tags() {
            num_seen += 1;
            if let Some(ptag) = previous_tag {
                if !(ptag < tag) {
                    panic!("ordering problem");
                }
            }
            previous_tag = Some(tag);
        }
        if num_seen != self.positions.len() as u64 {
            panic!("not all seen in iter");
        }
        /*
        // TODO maybe verify list integrity and then tag ordering? idk
        if let Some((value1, position1)) = self.positions.iter().next() {
            let mut value: T = value1.clone();
            let mut prev: T = position1.prev.clone();
            let mut next: T = position1.next.clone();
            let mut lowest_tag: Tag = position1.tag;
            let lowest_value = loop {
                if let Some(prev_position) = self.positions.get(&prev) {
                    if prev_position.tag < lowest_tag {
                        value = prev;
                        prev = prev_position.prev.clone();
                        next = prev_position.next.clone();
                        lowest_tag = prev_position.tag;
                        continue;
                    }
                }
                break value.clone();
            };
            let mut num_seen: u64 = 0;
            let mut tag: Tag = lowest_tag;
            loop {
                num_seen += 1;
                if let Some(next_position) = self.positions.get(&next) {
                    if next_position.prev != value {
                        panic!("integrity of prev/next");
                    }
                    if next == lowest_value {
                        break;
                    }
                    if tag >= next_position.tag {
                        panic!("sequencing bug");
                    }
                    tag = next_position.tag;
                    value = next;
                    next = next_position.next.clone();
                } else {
                    panic!("there should always be a next");
                }
            }
            if num_seen != self.positions.len() as u64 {
                panic!("not all seen");
            }
        }*/
    }
    fn rebalance(&mut self, value: &T) {
       let front = match self.front.clone() {None => return, Some(a) => a};
       let mut base_tag: Tag = self.positions.get(value).unwrap().tag;
       let mut mask: Tag = 0;
       let mut threshold: f64 = 1.0;
       let mut first: T = value.clone();
       let mut last: T = value.clone();
       let mut num_items: usize = 1;
       let multiplier: f64 = 2.0 / (2.0 * (self.len() as f64)).powf(1.0 / 62.0); // ??
       loop {
           {
               let mut prev: T;
               //let mut first_tag: Tag;
               {
                   let first_position = self.positions.get(&first).unwrap();
                   prev = first_position.prev.clone();
                   //first_tag = first_position.tag;
               }
               loop {
                   let prev_position = self.positions.get(&prev).unwrap();
                   let prev_tag = prev_position.tag;
                   if first != front && prev_tag &! mask == base_tag {
                       first = prev;
                       prev = prev_position.prev.clone();
                       //first_tag = prev_position.tag;
                       num_items += 1;
                   } else {
                       break;
                   }
               }
           }
           {
               let mut next: T;
               //let mut last_tag: Tag;
               {
                   let last_position = self.positions.get(&last).unwrap();
                   next = last_position.next.clone();
                   //last_tag = last_position.tag;
               }
               loop {
                   let next_position = self.positions.get(&next).unwrap();
                   let next_tag = next_position.tag;
                   if next != front && next_tag &! mask == base_tag {
                       last = next;
                       next = next_position.next.clone();
                       //last_tag = next_position.tag;
                       num_items += 1;
                   } else {
                       break;
                   }
               }
           }
           let increment = (mask + 1) / (num_items as Tag);
           if (increment as f64) >= threshold {
               let mut item = first;
               let mut new_tag = base_tag;
               while item != last {
                   let item_position = self.positions.get_mut(&item).unwrap();
                   item_position.tag = new_tag;
                   new_tag += increment;
                   item = item_position.next.clone();
               }
               self.positions.get_mut(&item).unwrap().tag = new_tag;
               return;
           }
           mask = (mask << 1) + 1;
           base_tag = base_tag &! mask;
           threshold *= multiplier;
       }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basically() {
        let mut om: OrderMaintenance<&'static str> = OrderMaintenance::new();
        assert_eq!(om.len(), 0);
        om.insert_only("bob");
        assert_eq!(om.len(), 1);
        om.insert_after(&"bob", "carol");
        assert_eq!(om.len(), 2);
        om.insert_after(&"bob", "james");
        assert_eq!(om.len(), 3);
        om.insert_after(&"carol", "gene");
        assert_eq!(om.len(), 4);
        assert_eq!(om.compare(&"bob", &"carol"), Some(Ordering::Less));
        assert_eq!(om.compare(&"bob", &"james"), Some(Ordering::Less));
        assert_eq!(om.compare(&"bob", &"gene"), Some(Ordering::Less));
        assert_eq!(om.compare(&"james", &"carol"), Some(Ordering::Less));
        assert_eq!(om.compare(&"james", &"gene"), Some(Ordering::Less));
        assert_eq!(om.compare(&"carol", &"gene"), Some(Ordering::Less));
        assert_eq!(om.compare(&"gene", &"gene"), Some(Ordering::Equal));
        assert_eq!(om.compare(&"carol", &"carol"), Some(Ordering::Equal));
        assert_eq!(om.compare(&"james", &"james"), Some(Ordering::Equal));
        assert_eq!(om.compare(&"bob", &"bob"), Some(Ordering::Equal));
        assert_eq!(om.compare(&"carol", &"james"), Some(Ordering::Greater));
    }
}


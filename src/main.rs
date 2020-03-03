use crate::Stay::{Empty, GuestId};
use std::collections::HashMap;

// This program schedules the bed occupation for an hotel.
// As input:
// - number of beds
// - the schedule of each guest start-end (the end of one guest can overlap with the start of another guest)
// We want to host the max. number of guests possible.

fn main() {
    // TODO parse input from a file for usage as a main
    let bed_count = 0;
    let guests = Vec::new();
    println!("Solving schedule for {} bed(s) and {} guests", bed_count, guests.len());
    let schedules = solve_bed_scheduling(bed_count, guests);
    println!("{:#?}", schedules)
}

fn solve_bed_scheduling(bed_count: usize, guests: Vec<Guest>) -> Vec<BedSchedule> {
   if guests.is_empty() || bed_count == 0 {
       Vec::new()
   } else {
       let mut bed_schedules: Vec<BedSchedule> = Vec::new();
       let mut remaining_guests: Vec<Guest> = guests;
       for b in 1..=bed_count {
           if remaining_guests.is_empty() {
               let bed_schedule = BedSchedule { bed_id: b, schedule: Vec::new() };
               bed_schedules.push(bed_schedule);
           } else {
               let (entry_point, adjacency_map) = build_schedules_graph(&remaining_guests);
               let longest = longest_path_in_graph(entry_point, &adjacency_map);
               for s in &longest {
                   match s {
                       Empty => (),
                       GuestId(id) => {
                           let index = remaining_guests.iter().position(|&g| g.id == *id).unwrap();
                           remaining_guests.remove(index);
                       }
                   }
               }
               let bed_schedule = BedSchedule { bed_id: b, schedule: longest };
               bed_schedules.push(bed_schedule);
           }
       }
       bed_schedules
   }
}

fn build_schedules_graph(guests: &[Guest]) -> (usize, HashMap<usize, Vec<(usize, Stay)>>) {
    let mut dates: Vec<usize> = guests.iter()
        .flat_map(|g| vec![g.start, g.end] )
        .collect();

    // remove duplicated dates
    dates.sort();
    dates.dedup();

    // indexing guests by dates for lookup
    let mut guest_by_start_stay: HashMap<usize, Vec<&Guest>> = HashMap::new();
    let mut guest_by_end_stay: HashMap<usize, Vec<&Guest>> = HashMap::new();
    for g in guests {
        guest_by_start_stay.entry(g.start)
            .or_insert_with(Vec::new)
            .push(g);

        guest_by_end_stay.entry(g.end)
            .or_insert_with(Vec::new)
            .push(g);
    }

    // date to next possible dates
    let mut adjacency_map: HashMap<usize, Vec<(usize, Stay)>> = HashMap::new();

    for d in &dates {
        match guest_by_start_stay.get(&d) {
            None => {
                adjacency_map.entry(*d).or_insert_with(Vec::new);
            },
            Some(guests) => {
                let mut possible_stays: Vec<(usize, Stay)> = guests.iter()
                    .map(|g| (g.end, GuestId(g.id)))
                    .collect();

                adjacency_map.entry(*d)
                    .or_insert_with(Vec::new)
                    .append(&mut possible_stays);
            }
        }
    }

    let mut orphan_dates: Vec<usize> = Vec::new();
    let start_date = dates.first().unwrap();
    for d in &dates {
        if d != start_date && !guest_by_end_stay.contains_key(&d) {
            orphan_dates.push(*d)
        }
    }
    // orphan dates need to be wired to the graph for exploration
    // we pick the previous date as anchor
    for o in orphan_dates {
        let index = dates.iter().position(|&d| d == o).unwrap();
        let prev = dates.get(index - 1).unwrap();
        adjacency_map.entry(*prev)
            .or_insert_with(Vec::new)
            .push((o, Empty))
    }

    (*start_date, adjacency_map)
}

fn longest_path_in_graph(entry_point: usize, adjacency_map : &HashMap<usize, Vec<(usize, Stay)>>) -> Vec<Stay> {

    fn recurse(entry_point: usize, adjacency_map : &HashMap<usize, Vec<(usize, Stay)>>, acc: Vec<Stay>) -> Vec<Stay> {
        let paths = adjacency_map.get(&entry_point).unwrap();
        if paths.is_empty() {
            acc
        } else {
            let mut all_paths: Vec<Vec<Stay>> = paths.iter()
                .map(|p| {
                    let mut tmp = acc.clone();
                    tmp.push(p.1);
                    recurse(p.0, adjacency_map, tmp)
                })
                .collect();

            // Sort by longest path containing guests
            all_paths.sort_by(|p1, p2| {
                let p1_guests: Vec<&Stay> = p1.iter().filter(|&&s| s != Empty).collect();
                let p2_guests: Vec<&Stay> = p2.iter().filter(|&&s| s != Empty).collect();
                p1_guests.len().cmp(&p2_guests.len())
            });
            all_paths.last().unwrap().clone()
        }
    }

    recurse(entry_point, adjacency_map, Vec::new())
}

#[derive(Debug, PartialEq, Clone, Copy)]
struct Guest {
    id: usize,
    start: usize,
    end: usize
}

impl Guest {
    fn new(id: usize, start: usize, end: usize) -> Guest {
       Guest{ id, start, end}
    }
}

#[derive(Debug, PartialEq)]
struct BedSchedule {
    bed_id: usize,
    schedule: Vec<Stay>
}

impl BedSchedule {
    fn new(bed_id: usize, schedule: Vec<Stay>) -> BedSchedule {
        BedSchedule{ bed_id, schedule}
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum Stay {
    GuestId(usize),
    Empty
}

#[cfg(test)]
mod tests {
    use crate::{solve_bed_scheduling, BedSchedule, Guest};
    use crate::Stay::GuestId;

    #[test]
    fn no_guests_no_schedules() {
        let schedules = solve_bed_scheduling(1, Vec::new());
        let expected: Vec<BedSchedule> = Vec::new();
        assert_eq!(schedules, expected);
    }

    #[test]
    fn take_all() {
        // [ [1, 5], [5, 9], [9, 11], [11, 12] ]
        let guests = vec![
            Guest::new(1, 1, 5),
            Guest::new(2, 5, 9),
            Guest::new(3, 9, 11),
            Guest::new(4, 11, 12)
        ];
        let schedules = solve_bed_scheduling(1, guests);
        let expected: Vec<BedSchedule> = vec![
          BedSchedule::new(1, vec![GuestId(1), GuestId(2), GuestId(3), GuestId(4)])
        ];
        assert_eq!(schedules, expected);
    }

    #[test]
    fn skip_if_not_optimal() {
        // [ [1, 5], [5, 9], [8, 10], [9, 11], [11, 12] ]
        let guests = vec![
            Guest::new(1, 1, 5),
            Guest::new(2, 5, 9),
            Guest::new(3, 8, 10),
            Guest::new(4, 9, 11),
            Guest::new(5, 11, 12)
        ];
        let schedules = solve_bed_scheduling(1, guests);
        let expected: Vec<BedSchedule> = vec![
            BedSchedule::new(1, vec![GuestId(1), GuestId(2), GuestId(4), GuestId(5)])
        ];
        assert_eq!(schedules, expected);
    }

    #[test]
    fn two_beds_skip() {
        // [ [1, 5], [5, 9], [8, 10], [9, 11], [11, 12] ]
        let guests = vec![
            Guest::new(1, 1, 5),
            Guest::new(2, 5, 9),
            Guest::new(3, 8, 10),
            Guest::new(4, 9, 11),
            Guest::new(5, 11, 12)
        ];
        let schedules = solve_bed_scheduling(2, guests);
        let expected: Vec<BedSchedule> = vec![
            BedSchedule::new(1, vec![GuestId(1), GuestId(2), GuestId(4), GuestId(5)]),
            BedSchedule::new(2, vec![GuestId(3)])
        ];
        assert_eq!(schedules, expected);
    }

    #[test]
    fn empty_beds_possible() {
        // [ [1, 5], [5, 9], [8, 10], [9, 11], [11, 12] ]
        let guests = vec![
            Guest::new(1, 1, 5),
            Guest::new(2, 5, 9),
            Guest::new(3, 8, 10),
            Guest::new(4, 9, 11),
            Guest::new(5, 11, 12)
        ];
        let schedules = solve_bed_scheduling(3, guests);
        let expected: Vec<BedSchedule> = vec![
            BedSchedule::new(1, vec![GuestId(1), GuestId(2), GuestId(4), GuestId(5)]),
            BedSchedule::new(2, vec![GuestId(3)]),
            BedSchedule::new(3, Vec::new())
        ];
        assert_eq!(schedules, expected);
    }

}
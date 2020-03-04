use crate::Stay::{Empty, GuestId};
use std::collections::HashMap;

fn main() {
    // TODO parse input from a file for usage as a main
    let bed_count = 0;
    let guests = Vec::new();
    println!(
        "Solving schedule for {} bed(s) and {} guests",
        bed_count,
        guests.len()
    );
    let schedules = solve_bed_scheduling(bed_count, guests);
    println!("{:#?}", schedules)
}

fn solve_bed_scheduling(bed_count: usize, guests: Vec<Guest>) -> Vec<BedSchedule> {
    if guests.is_empty() || bed_count == 0 {
        Vec::new()
    } else {
        let mut bed_schedules: Vec<BedSchedule> = Vec::new();
        let mut remaining_guests: Vec<Guest> = guests;
        // TODO replace by a fold if possible
        for b in 1..=bed_count {
            if remaining_guests.is_empty() {
                let bed_schedule = BedSchedule {
                    bed_id: b,
                    schedule: Vec::new(),
                };
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
                let bed_schedule = BedSchedule {
                    bed_id: b,
                    schedule: longest,
                };
                bed_schedules.push(bed_schedule);
            }
        }
        bed_schedules
    }
}

fn build_schedules_graph(guests: &[Guest]) -> (usize, HashMap<usize, Vec<(usize, Stay)>>) {
    let mut dates: Vec<usize> = guests.iter().flat_map(|g| vec![g.start, g.end]).collect();
    // remove duplicated dates
    dates.sort();
    dates.dedup();

    // indexing guests by dates for lookup
    let mut guest_by_start_stay: HashMap<usize, Vec<&Guest>> = HashMap::new();
    let mut guest_by_end_stay: HashMap<usize, Vec<&Guest>> = HashMap::new();
    for g in guests {
        guest_by_start_stay
            .entry(g.start)
            .or_insert_with(Vec::new)
            .push(g);

        guest_by_end_stay
            .entry(g.end)
            .or_insert_with(Vec::new)
            .push(g);
    }

    // date to next possible dates
    let mut adjacency_map: HashMap<usize, Vec<(usize, Stay)>> = HashMap::new();
    for d in &dates {
        match guest_by_start_stay.get(&d) {
            None => {
                adjacency_map.entry(*d).or_insert_with(Vec::new);
            }
            Some(guests) => {
                let mut possible_stays: Vec<(usize, Stay)> =
                    guests.iter().map(|g| (g.end, GuestId(g.id))).collect();

                adjacency_map
                    .entry(*d)
                    .or_insert_with(Vec::new)
                    .append(&mut possible_stays);
            }
        }
    }

    // orphan dates need to be wired to the graph to enable exploration
    let mut orphan_dates: Vec<usize> = Vec::new();
    let start_date = dates.first().unwrap();
    for d in &dates {
        if d != start_date && !guest_by_end_stay.contains_key(&d) {
            orphan_dates.push(*d)
        }
    }
    // we pick the previous date as anchor
    for o in orphan_dates {
        let index = dates.iter().position(|&d| d == o).unwrap();
        let prev = dates.get(index - 1).unwrap();
        adjacency_map
            .entry(*prev)
            .or_insert_with(Vec::new)
            .push((o, Empty))
    }

    (*start_date, adjacency_map)
}

fn longest_path_in_graph(
    entry_point: usize,
    adjacency_map: &HashMap<usize, Vec<(usize, Stay)>>,
) -> Vec<Stay> {
    fn recurse(
        entry_point: usize,
        adjacency_map: &HashMap<usize, Vec<(usize, Stay)>>,
        acc: Vec<Stay>,
    ) -> Vec<Stay> {
        let paths = adjacency_map.get(&entry_point).unwrap();
        if paths.is_empty() {
            acc
        } else {
            let mut all_paths: Vec<Vec<Stay>> = paths
                .iter()
                .map(|(date, stay)| {
                    match stay {
                        Empty => recurse(*date, adjacency_map, acc.clone()), //discard empty stay
                        GuestId(_) => {
                            let mut fork = acc.clone();
                            fork.push(*stay);
                            recurse(*date, adjacency_map, fork)
                        }
                    }
                })
                .collect();

            // Sort by longest paths guests
            all_paths.sort_by(|p1, p2| p1.len().cmp(&p2.len()));
            all_paths.last().unwrap().to_vec()
        }
    }

    recurse(entry_point, adjacency_map, Vec::new())
}

#[derive(Debug, PartialEq, Clone, Copy)]
struct Guest {
    id: usize,
    start: usize,
    end: usize,
}

impl Guest {
    fn new(id: usize, start: usize, end: usize) -> Guest {
        Guest { id, start, end }
    }
}

#[derive(Debug, PartialEq)]
struct BedSchedule {
    bed_id: usize,
    schedule: Vec<Stay>,
}

impl BedSchedule {
    fn new(bed_id: usize, schedule: Vec<Stay>) -> BedSchedule {
        BedSchedule { bed_id, schedule }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum Stay {
    GuestId(usize),
    Empty,
}

#[cfg(test)]
mod tests {
    use crate::Stay::*;
    use crate::{solve_bed_scheduling, BedSchedule, Guest};

    #[test]
    fn no_guests_no_schedules() {
        let schedules = solve_bed_scheduling(1, Vec::new());
        let expected: Vec<BedSchedule> = Vec::new();
        assert_eq!(schedules, expected);
    }

    #[test]
    fn overlapping_guests_use_different_beds() {
        let guests = vec![
            Guest::new(1, 1, 5),
            Guest::new(2, 1, 5),
            Guest::new(3, 5, 10),
        ];
        let schedules = solve_bed_scheduling(2, guests);
        let expected: Vec<BedSchedule> = vec![
            BedSchedule::new(1, vec![GuestId(2), GuestId(3)]),
            BedSchedule::new(2, vec![GuestId(1)]),
        ];
        assert_eq!(schedules, expected);
    }

    #[test]
    fn schedule_all_guests_in_single_bed_if_no_overlap() {
        let guests = vec![
            Guest::new(1, 1, 5),
            Guest::new(2, 5, 9),
            Guest::new(3, 9, 11),
            Guest::new(4, 11, 12),
        ];
        let schedules = solve_bed_scheduling(1, guests);
        let expected: Vec<BedSchedule> = vec![BedSchedule::new(
            1,
            vec![GuestId(1), GuestId(2), GuestId(3), GuestId(4)],
        )];
        assert_eq!(schedules, expected);
    }

    #[test]
    fn maximise_number_of_host_per_bed() {
        let guests = vec![
            Guest::new(1, 1, 5),
            Guest::new(2, 5, 9),
            Guest::new(3, 8, 10),
            Guest::new(4, 9, 11),
            Guest::new(5, 11, 12),
        ];
        let schedules = solve_bed_scheduling(1, guests);
        let expected: Vec<BedSchedule> = vec![BedSchedule::new(
            1,
            vec![GuestId(1), GuestId(2), GuestId(4), GuestId(5)],
        )];
        assert_eq!(schedules, expected);
    }

    #[test]
    fn maximise_number_of_host_per_bed_bis() {
        let guests = vec![
            Guest::new(1, 1, 5),
            Guest::new(2, 5, 9),
            Guest::new(3, 8, 10),
            Guest::new(4, 9, 11),
            Guest::new(5, 11, 12),
        ];
        let schedules = solve_bed_scheduling(2, guests);
        let expected: Vec<BedSchedule> = vec![
            BedSchedule::new(1, vec![GuestId(1), GuestId(2), GuestId(4), GuestId(5)]),
            BedSchedule::new(2, vec![GuestId(3)]),
        ];
        assert_eq!(schedules, expected);
    }

    #[test]
    fn empty_beds_possible() {
        let guests = vec![
            Guest::new(1, 1, 5),
            Guest::new(2, 5, 9),
            Guest::new(3, 8, 10),
            Guest::new(4, 9, 11),
            Guest::new(5, 11, 12),
        ];
        let schedules = solve_bed_scheduling(3, guests);
        let expected: Vec<BedSchedule> = vec![
            BedSchedule::new(1, vec![GuestId(1), GuestId(2), GuestId(4), GuestId(5)]),
            BedSchedule::new(2, vec![GuestId(3)]),
            BedSchedule::new(3, Vec::new()),
        ];
        assert_eq!(schedules, expected);
    }

    #[test]
    fn maximises_number_hosts_over_bed_occupation_one_bed() {
        let guests = vec![
            Guest::new(1, 1, 10),
            Guest::new(2, 10, 20),
            Guest::new(3, 1, 5),
            Guest::new(4, 5, 10),
            Guest::new(5, 10, 15),
            Guest::new(6, 15, 20),
        ];
        let schedules = solve_bed_scheduling(1, guests);
        let expected: Vec<BedSchedule> = vec![BedSchedule::new(
            1,
            vec![GuestId(3), GuestId(4), GuestId(5), GuestId(6)],
        )];
        assert_eq!(schedules, expected);
    }

    #[test]
    fn maximises_number_hosts_over_bed_occupation_two_beds() {
        let guests = vec![
            Guest::new(1, 1, 10),
            Guest::new(2, 10, 20),
            Guest::new(3, 1, 5),
            Guest::new(4, 5, 10),
            Guest::new(5, 10, 15),
            Guest::new(6, 15, 20),
        ];
        let schedules = solve_bed_scheduling(2, guests);
        let expected: Vec<BedSchedule> = vec![
            BedSchedule::new(1, vec![GuestId(3), GuestId(4), GuestId(5), GuestId(6)]),
            BedSchedule::new(2, vec![GuestId(1), GuestId(2)]),
        ];
        assert_eq!(schedules, expected);
    }

    #[test]
    fn all_guests_fit_in_one_bed_without_overlap() {
        let max = 1000;
        let mut guests: Vec<Guest> = Vec::new();
        for i in 1..=max {
            let g = Guest::new(i, i, i + 1);
            guests.push(g);
        }
        let expected_scheduled_guests: Vec<usize> = guests.iter().map(|g| g.id).collect();
        let schedules = solve_bed_scheduling(1, guests);
        assert_eq!(schedules.len(), 1);
        let first_schedule = schedules.first().unwrap();
        let scheduled_guests: Vec<usize> = first_schedule
            .schedule
            .iter()
            .filter_map(|s| match s {
                Empty => None,
                GuestId(id) => Some(*id),
            })
            .collect();
        assert_eq!(scheduled_guests, expected_scheduled_guests);
    }

    // TODO add property based tests with Quickcheck with generated guests to validate the invariants:
    // - all guests are hosted if no overlap in guests dates
    // - a single bed is enough if no overlap in guests dates
    // - number of hosts in bed `b` is always >= to number of hosts in bed `b+1`
    // - the number of beds required to hosts any list of guests is equal
    //   to the highest number of guests having an overlap on a given date
    //   e.g 3 guests overlapping on the same date would need 3 different beds
}

use std::cmp;
use std::mem;
use std::f64;

extern crate time;

type StreamType = f64;

const HLINE: &'static str = "-------------------------------------------------------------\n";
const STREAM_ARRAY_SIZE: usize = 10000000;
const NTIMES: usize = 10;
const M: usize = 10;

static mut A: [StreamType; STREAM_ARRAY_SIZE] = [0.0; STREAM_ARRAY_SIZE];
static mut B: [StreamType; STREAM_ARRAY_SIZE] = [0.0; STREAM_ARRAY_SIZE];
static mut C: [StreamType; STREAM_ARRAY_SIZE] = [0.0; STREAM_ARRAY_SIZE];

fn main() {
    /* --- SETUP --- determine precision and check timing --- */
    println!("{}", HLINE);
    println!("STREAM version $Revision: 5.10 $");
    println!("{}", HLINE);
    let bytes_per_word = mem::size_of::<StreamType>();
    println!("This system uses {} bytes per array element.", bytes_per_word);

    println!("{}", HLINE);

    println!("Array size = {} (elements)", STREAM_ARRAY_SIZE);
    println!("Memory per array = {:.1} MiB (= {:.1} GiB).",
             bytes_per_word as f64 * (STREAM_ARRAY_SIZE as f64 / 1024.0 / 1024.0),
             bytes_per_word as f64 * (STREAM_ARRAY_SIZE as f64 / 1024.0 / 1024.0 / 1024.0));
    println!("Total memory required = {:.1} MiB (= {:.1} GiB).",
            (3.0 * bytes_per_word as f64) * (STREAM_ARRAY_SIZE as f64 / 1024.0 / 1024.0),
            (3.0 * bytes_per_word as f64) * (STREAM_ARRAY_SIZE as f64 / 1024.0/1024./1024.0));
    println!("Each kernel will be executed {} times.", NTIMES);
    println!(" The *best* time for each kernel (excluding the first iteration)");
    println!(" will be used to compute the reported bandwidth.");
    
	println!("{}", HLINE);

    unsafe {
        for i in 0..STREAM_ARRAY_SIZE {
            A[i] = 1.0;
            B[i] = 2.0;
            C[i] = 0.0;
        }
    }

    let mut quantum = check_tick();
    if  quantum >= 1 {
        println!("Your clock granularity/precision appears to be {} microseconds.", quantum);
    } else {
	    println!("Your clock granularity appears to be less than one microsecond.");
	    quantum = 1;
    }

    let mut t = time::precise_time_s();
    unsafe {
        for i in 0..STREAM_ARRAY_SIZE {
            A[i] = 2.0 * A[i];
        }
    }
    t = 1.0E6 * (time::precise_time_s() - t);

    println!("Each test below will take on the order of {} microseconds.", t as i32);
    println!("   (= {} clock ticks)", (t / quantum as f64) as i32);
    println!("Increase the size of the arrays if this shows that");
    println!("you are not getting at least 20 clock ticks per test.");

    println!("{}", HLINE);

    println!("WARNING -- The above is only a rough guideline.");
    println!("For best results, please be sure you know the");
    println!("precision of your system timer.");
    println!("{}", HLINE);

    let mut times: [[f64; NTIMES]; 4] = [[0.0; NTIMES]; 4];

    /*	--- MAIN LOOP --- repeat test cases NTIMES times --- */

    let scalar:StreamType = 3.0;
    for k in 0..NTIMES {
        times[0][k] = time::precise_time_s();
        unsafe {
            for j in 0..STREAM_ARRAY_SIZE {
                C[j] = A[j];
            }
        }
        times[0][k] = time::precise_time_s() - times[0][k];

        times[1][k] = time::precise_time_s();
        unsafe {
            for j in 0..STREAM_ARRAY_SIZE {
                B[j] = scalar * C[j];
            }
        }
        times[1][k] = time::precise_time_s() - times[1][k];

        times[2][k] = time::precise_time_s();
        unsafe {
            for j in 0..STREAM_ARRAY_SIZE {
                C[j] = A[j] + B[j];
            }
        }
        times[2][k] = time::precise_time_s() - times[2][k];

        times[3][k] = time::precise_time_s();
        unsafe {
            for j in 0..STREAM_ARRAY_SIZE {
                A[j] = B[j] + scalar * C[j];
            }
        }
        times[3][k] = time::precise_time_s() - times[3][k];
    }

    let mut avg_time: [f64; 4] = [0.0; 4];
    let mut max_time: [f64; 4] = [0.0; 4];
    let mut min_time: [f64; 4] = [f64::MAX, f64::MAX, f64::MAX, f64::MAX];

    let label: [&'static str; 4] = ["Copy:      ", "Scale:     ",  "Add:       ", "Triad:     "];
    let bytes: [usize; 4] = [
        2 * bytes_per_word * STREAM_ARRAY_SIZE ,
        2 * bytes_per_word * STREAM_ARRAY_SIZE ,
        3 * bytes_per_word * STREAM_ARRAY_SIZE ,
        3 * bytes_per_word * STREAM_ARRAY_SIZE
    ];

    /*	--- SUMMARY --- */

    for k in 1..NTIMES /* note -- skip first iteration */
    {
        for j in 0..4
        {
            avg_time[j] = avg_time[j] + times[j][k];
            if min_time[j] > times[j][k] {
                min_time[j] = times[j][k];
            }
            if max_time[j] < times[j][k] {
                max_time[j] = times[j][k];
            }
        }
    }

    println!("Function    Best Rate MB/s  Avg time     Min time     Max time");
    for j in 0..4 {
        avg_time[j] = avg_time[j] / (NTIMES-1) as f64;

        println!("{}{:12.1}  {:11.6}  {:11.6}  {:11.6}", label[j],
            1.0E-06 * bytes[j] as f64 / min_time[j],
            avg_time[j],
            min_time[j],
            max_time[j]);
    }
    println!("{}", HLINE);

    check_stream_results();
    println!("{}", HLINE);
}

fn check_tick() -> i32 {
    let mut times_found: [f64; M] = [0.0; M];

    for i in 0..M {
        let t1 = time::precise_time_s();

        loop {
            let t2 = time::precise_time_s();
            if (t2 - t1) >= 1.0E-6 {
                times_found[i] = t2;
                break;
            }
        }
    }

    let mut min_delta = 1000000;
    for i in 1..M {
        let delta = (1.0E6 * (times_found[i] - times_found[i-1])) as i32;
        min_delta = cmp::min(min_delta, cmp::max(delta, 0));
    }

    min_delta

}

fn check_stream_results() {
    /* reproduce initialization */
    let mut aj: StreamType = 1.0;
    let mut bj: StreamType = 2.0;
    let mut cj: StreamType = 0.0;
    /* a[] is modified during timing check */
    aj = 2.0E0 * aj;
    /* now execute timing loop */
    let scalar: StreamType = 3.0;
    for _ in 0..NTIMES {
        cj = aj;
        bj = scalar * cj;
        cj = aj + bj;
        aj = bj + scalar * cj;
    }

    /* accumulate deltas between observed and expected results */
    let mut a_sum_err: StreamType = 0.0;
    let mut b_sum_err: StreamType = 0.0;
    let mut c_sum_err: StreamType = 0.0;
    for j in 0..STREAM_ARRAY_SIZE {
        unsafe {
            a_sum_err += (A[j] - aj).abs();
            b_sum_err += (B[j] - bj).abs();
            c_sum_err += (C[j] - cj).abs();
        }
    }
    let a_avg_err = a_sum_err / STREAM_ARRAY_SIZE as f64;
    let b_avg_err = b_sum_err / STREAM_ARRAY_SIZE as f64;
    let c_avg_err = c_sum_err / STREAM_ARRAY_SIZE as f64;

    let epsilon = 1.0E-13;

    let mut err = 0;
    let mut ierr = 0;

    if (a_avg_err / aj).abs() > epsilon {
        err = err + 1;
        println!("Failed Validation on array a[], AvgRelAbsErr > epsilon ({})", epsilon);
        println!("     Expected Value: {}, AvgAbsErr: {}, AvgRelAbsErr: {}",aj, a_avg_err, a_avg_err.abs()/aj);
        for j in 0..STREAM_ARRAY_SIZE {
            unsafe {
                if (A[j] / aj - 1.0).abs() > epsilon {
                    ierr = ierr + 1;
                }
            }
        }
        println!("     For array a[], {} errors were found.", ierr);
    }

    if (b_avg_err / bj).abs() > epsilon {
        err = err + 1;
        println!("Failed Validation on array b[], AvgRelAbsErr > epsilon ({})", epsilon);
        println! ("     Expected Value: {}, AvgAbsErr: {}, AvgRelAbsErr: {}",bj, b_avg_err, b_avg_err.abs()/bj);
        println! ("     AvgRelAbsErr > Epsilon {}", epsilon);
        ierr = 0;
        for j in 0..STREAM_ARRAY_SIZE {
            unsafe {
                if (B[j]/ bj - 1.0).abs() > epsilon {
                    ierr = ierr + 1;
                }
            }
        }
        println!("     For array b[], {} errors were found.", ierr);
    }

    if (c_avg_err / cj).abs() > epsilon {
        err = err + 1;
        println! ("Failed Validation on array c[], AvgRelAbsErr > epsilon ({})", epsilon);
        println! ("     Expected Value: {}, AvgAbsErr: {}, AvgRelAbsErr: {}",cj, c_avg_err, c_avg_err.abs() / cj);
        println! ("     AvgRelAbsErr > Epsilon ({})",epsilon);
        ierr = 0;
        for j in 0..STREAM_ARRAY_SIZE {
            unsafe {
                if (C[j] / cj - 1.0).abs() > epsilon {
                    ierr = ierr + 1;
                }
            }
        }
        println!("     For array c[], {} errors were found.", ierr);
    }

    if err == 0 {
        println! ("Solution Validates: avg error less than {:e} on all three arrays", epsilon);
    }

}


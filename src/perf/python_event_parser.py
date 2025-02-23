import sys
import re

def aggregate_perf_events_python(perf_script_output_file: str, function_name: str):
    event_counts = {
        "L1-dcache-loads": 0,
        "L1-dcache-load-misses": 0,
        "branch-instructions": 0,
        "branch-misses": 0,
    }

    with open(perf_script_output_file, 'r') as f:
        for line in f:
            line = line.strip()
            if not line:
                continue

            parts = re.split(r"\:\s+|\s+", line)
            func_name = parts[6]

            match = re.search(r'([^:]+?::)*?([^:+]+)(\+.*)?$', func_name)


            if match:
                matched_name = match.group(2)
                if matched_name == function_name:

                    event_count = int(parts[3])
                    evente_name = parts[4]

                    if evente_name in event_counts:
                        event_counts[evente_name] += event_count

    return event_counts
                    

if __name__ == "__main__":
    if len(sys.argv) != 3:
        print("Usage: python python_event_parser.py <perf_script_output_file> <function name>")
        sys.exit(1)

    perf_script_file = sys.argv[1]
    function_name = sys.argv[2]
    aggregated_events = aggregate_perf_events_python(perf_script_file, function_name)



    l1_perc = None

    if "L1-dcache-loads" and "L1-dcache-load-misses" in aggregated_events:
        l1_perc = aggregated_events["L1-dcache-load-misses"] / aggregated_events["L1-dcache-loads"] * 100
        l1_perc = round(l1_perc, 2)

    for event, count in aggregated_events.items():
        match event:
            case "L1-dcache-load-misses":
                print(f"{count:>15,}     {event:<30}     #   {l1_perc:>6.2f}% of all L1 cache accesses")
            case _:
                print(f"{count:>15,}     {event:<30}")

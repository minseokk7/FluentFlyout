import os
import glob

log_pattern = os.path.join(os.environ['APPDATA'], 'FluentFlyout', 'logs.*.txt')
logs = glob.glob(log_pattern)
if not logs:
    with open('log_tail.txt', 'w', encoding='utf-8') as f:
        f.write("No logs found.")
else:
    latest_log = max(logs, key=os.path.getmtime)
    with open(latest_log, 'r', encoding='utf-8') as f:
        lines = f.readlines()
        with open('log_tail.txt', 'w', encoding='utf-8') as out:
            out.write(f"Reading: {latest_log}\n")
            for line in lines[-100:]:
                out.write(line)

import subprocess

result = subprocess.run(
    ["dotnet", "build", r".\FluentFlyoutWPF\FluentFlyout.csproj", "--configuration", "Debug"],
    capture_output=True,
    encoding="utf-8",
    errors="replace",
    cwd=r"c:\Users\minse\.gemini\antigravity\playground\spatial-kilonova\FluentFlyout"
)

with open("build_output.txt", "w", encoding="utf-8") as f:
    f.write("=== STDOUT ===\n")
    f.write(result.stdout)
    f.write("\n=== STDERR ===\n")
    f.write(result.stderr)
    f.write(f"\nReturn code: {result.returncode}\n")

# 에러 라인만 출력
for line in result.stdout.splitlines():
    if "error" in line.lower():
        print(line)
print(f"Return code: {result.returncode}")

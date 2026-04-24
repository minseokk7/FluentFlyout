import subprocess

def run_cargo(target_dir):
    try:
        result = subprocess.run(
            ['cargo', 'check'],
            cwd=target_dir,
            capture_output=True,
            text=True,
            encoding='utf-8',
            errors='ignore'
        )
        
        with open('rust_errors.txt', 'w', encoding='utf-8') as f:
            f.write("--- STDOUT ---\n")
            f.write(result.stdout)
            f.write("\n--- STDERR ---\n")
            f.write(result.stderr)
            
        # 실행 결과 상태를 콘솔에 출력
        if result.returncode == 0:
            print("✅ 'cargo check'가 성공적으로 완료되었습니다. (에러 없음)")
        else:
            print(f"❌ 'cargo check' 실행 중 문제가 발견되었습니다. (종료 코드: {result.returncode})")
        
        print("상세 내용은 'rust_errors.txt' 파일을 확인해 주세요.")
            
    except FileNotFoundError:
        print("Error: 'cargo' 명령어를 찾을 수 없습니다. 시스템에 Rust가 설치되어 있는지 확인해 주세요.")
    except Exception as e:
        print(f"Unexpected Error: {e}")

if __name__ == "__main__":
    # 작업할 Rust 프로젝트 디렉토리 경로 지정
    project_path = r'c:\Users\minse\.gemini\antigravity\playground\spatial-kilonova\FluentFlyout\fluent_flyout_core'
    run_cargo(project_path)

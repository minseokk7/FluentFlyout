import os

if os.path.exists('build_err.txt'):
    with open('build_err.txt', 'rb') as f:
        content = f.read()
    # Try different encodings
    for encoding in ['utf-16', 'utf-8', 'cp949']:
        try:
            text = content.decode(encoding)
            print(f"Decoded with {encoding}:")
            print(text)
            break
        except:
            continue
else:
    print("build_err.txt not found.")

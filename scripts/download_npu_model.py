# pylint: skip-file
"""
HMIR NPU Model Downloader.
Downloads OpenVINO-compatible models from Hugging Face for the Intel NPU.
"""

import os
import sys
from huggingface_hub import snapshot_download  # pylint: disable=import-error

def get_data_dir():
    """Get the OS-appropriate storage directory for HMIR models."""
    if sys.platform == "win32":
        base = os.environ.get("LOCALAPPDATA", os.path.expanduser("~\\AppData\\Local"))
    elif sys.platform == "darwin":
        base = os.path.expanduser("~/Library/Application Support")
    else:
        base = os.environ.get("XDG_DATA_HOME", os.path.expanduser("~/.local/share"))
    return os.path.join(base, "hmir", "models")

def main():
    """Download and configure the requested model."""
    repo_id = sys.argv[1] if len(sys.argv) > 1 else "OpenVINO/qwen2.5-1.5b-instruct-int4-ov"
    folder_name = sys.argv[2] if len(sys.argv) > 2 else "qwen2.5-1.5b-ov"

    target_dir = os.path.join(get_data_dir(), folder_name)
    print(f"STARTING DOWNLOAD: {repo_id} into {folder_name}")

    try:
        snapshot_download(
            repo_id=repo_id,
            local_dir=target_dir,
            local_dir_use_symlinks=False
        )
        print(f"SUCCESS: Model downloaded to {target_dir}")
    except RuntimeError as e:
        print(f"ERROR: Model runtime issue during download: {e}")
        sys.exit(1)
    except (TypeError, ValueError, AttributeError) as e:
        print(f"ERROR: Configuration issue: {e}")
        sys.exit(1)
    except Exception as e:  # pylint: disable=broad-except
        print(f"ERROR: Unexpected download failure: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main()

import subprocess


def run_command(cmd):
    """Run a command without a shell.

    ``cmd`` must be a list of arguments. Never pass unvalidated user input
    to this function.
    """
    if isinstance(cmd, str):
        raise ValueError("cmd must be a list of arguments, not a string")
    result = subprocess.run(cmd, shell=False, capture_output=True, text=True)
    return result

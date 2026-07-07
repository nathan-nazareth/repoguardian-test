import subprocess

def run_command(cmd):
    result = subprocess.call(cmd, shell=True)
    return result

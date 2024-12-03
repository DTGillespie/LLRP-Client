import time
import os
import threading

def stream_log_file(file_path, polling_interval=1.0):

    if not os.path.exists(file_path):
        try:
            with open(file_path, 'w') as file:
                pass
            print(f"Log File '{file_path}' Created.")
        except Exception as e:
            print(f"Error creating log file '{file_path}': {e}")
            return

    try:
        with open(file_path, 'r') as file:
            file.seek(0, 2)
            
            print(f"Streaming Log File: {file_path}")
            print("-" * 50)

            while True:
                line = file.readline()
                if line:
                    print(line, end='')
                else:
                    time.sleep(polling_interval)
    except PermissionError:
        print(f"Error: Insufficient permissions to read '{file_path}'.")
    except KeyboardInterrupt:
        print("\nStreaming interrupted by user.")
    except Exception as e:
        print(f"An unexpected error occurred: {e}")

def clear_screen_listener():

    print("Press 'c' to clear the screen or Ctrl+C to exit.")
    try:
        while True:
            user_input = input()
            if user_input.lower() == 'c':
                os.system('cls' if os.name == 'nt' else 'clear')
    except KeyboardInterrupt:
        pass

if __name__ == "__main__":

    log_file_path = "system.log"

    listener_thread = threading.Thread(target=clear_screen_listener, daemon=True)
    listener_thread.start()

    stream_log_file(log_file_path)

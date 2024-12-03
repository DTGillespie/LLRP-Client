import time
import os
import threading

def stream_log_file(file_path, polling_interval=1.0):
    """
    Streams the contents of a log file to the terminal in real-time.

    :param file_path: Path to the log file to be streamed.
    :param polling_interval: Time interval (in seconds) between file checks.
    """
    try:
        with open(file_path, 'r') as file:
            # Move the pointer to the end of the file
            file.seek(0, 2)
            
            print(f"Streaming Log File: {file_path}")
            print("-" * 50)

            while True:
                # Read new lines from the file
                line = file.readline()
                if line:
                    print(line, end='')  # Print without adding extra newlines
                else:
                    time.sleep(polling_interval)  # Wait before checking again
    except FileNotFoundError:
        print(f"Error: File '{file_path}' not found.")
    except PermissionError:
        print(f"Error: Insufficient permissions to read '{file_path}'.")
    except KeyboardInterrupt:
        print("\nStreaming interrupted by user.")
    except Exception as e:
        print(f"An unexpected error occurred: {e}")

def clear_screen_listener():
    """
    Listens for the user to press the 'c' key and clears the terminal screen.
    """
    print("Press 'c' to clear the screen or Ctrl+C to exit.")
    try:
        while True:
            user_input = input()
            if user_input.lower() == 'c':
                os.system('cls' if os.name == 'nt' else 'clear')
    except KeyboardInterrupt:
        pass  # Gracefully handle Ctrl+C here if desired

if __name__ == "__main__":
    # Specify the path to the log file
    log_file_path = "system.log"  # Replace with the path to your log file

    # Start the clear screen listener in a separate thread
    listener_thread = threading.Thread(target=clear_screen_listener, daemon=True)
    listener_thread.start()

    # Start streaming the log file
    stream_log_file(log_file_path)

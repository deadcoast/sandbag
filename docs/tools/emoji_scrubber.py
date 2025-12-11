#!/usr/bin/env python3
"""
Emoji Scrubber - Remove emojis from files and directories
A quick, interactive, and aesthetically pleasing tool for cleaning documentation.
"""

import re
import sys
from pathlib import Path


class EmojiScrubber:
    """Advanced emoji removal tool with directory support and ignore system"""

    def __init__(self):
        self.ignore_patterns = {
            ".venv",
            "__pycache__",
            ".git",
            "node_modules",
            ".DS_Store",
            ".vscode",
            ".idea",
            "*.pyc",
            "*.pyo",
            "*.pyd",
            "*.so",
            "*.dll",
            "*.exe",
            "*.app",
            "*.dmg",
            "*.pkg",
            "*.zip",
            "*.tar.gz",
            "*.rar",
            "*.7z",
            "*.log",
            "*.tmp",
            "*.temp",
            "*.cache",
            "*.min.js",
            "*.min.css",
            "*.map",
            "package-lock.json",
            "yarn.lock",
            "poetry.lock",
        }

        # Comprehensive emoji regex pattern
        self.emoji_pattern = re.compile(
            r"["
            "\U0001f600-\U0001f64f"  # Emoticons
            "\U0001f300-\U0001f5ff"  # Symbols & Pictographs
            "\U0001f680-\U0001f6ff"  # Transport & Map Symbols
            "\U0001f1e0-\U0001f1ff"  # Flags
            "\U00002702-\U000027b0"  # Dingbats
            "\U000024c2-\U0001f251"  # Enclosed Characters
            "\U0001f900-\U0001f9ff"  # Supplemental Symbols
            "\U00002600-\U000026ff"  # Miscellaneous Symbols
            "\U00002700-\U000027bf"  # Dingbats Extended
            "\U0001f3fb-\U0001f3ff"  # Skin Tone Modifiers
            "\U0001f9b0-\U0001f9b3"  # Hair Components
            r"]+",
            flags=re.UNICODE,
        )

        self.stats = {"files_processed": 0, "files_modified": 0, "files_skipped": 0, "emojis_removed": 0, "errors": 0}

    def print_border(self, text: str, style: str = "default") -> None:
        """Print text inside an advanced ASCII border with different styles"""
        lines = text.split("\n")
        max_length = max(len(line) for line in lines) if lines else 0

        if style == "welcome":
            border_top = "" + "" * (max_length + 2) + ""
            border_bottom = "" + "" * (max_length + 2) + ""
        elif style == "success":
            border_top = "" + "" * (max_length + 2) + ""
            border_bottom = "" + "" * (max_length + 2) + ""
        elif style == "error":
            border_top = "" + "" * (max_length + 2) + ""
            border_bottom = "" + "" * (max_length + 2) + ""
        elif style == "info":
            border_top = "" + "" * (max_length + 2) + ""
            border_bottom = "" + "" * (max_length + 2) + ""
        else:
            border_top = "" + "" * (max_length + 2) + ""
            border_bottom = "" + "" * (max_length + 2) + ""

        print(border_top)
        for line in lines:
            print(" " + line.ljust(max_length) + " ")
        print(border_bottom)

    def print_progress_bar(self, current: int, total: int, description: str = "") -> None:
        """Display a beautiful progress bar"""
        bar_length = 40
        filled_length = int(bar_length * current // total)
        bar = "" * filled_length + "" * (bar_length - filled_length)
        percentage = current / total * 100

        print(f"\r{description} [{bar}] {percentage:.1f}% ({current}/{total})", end="", flush=True)
        if current == total:
            print()

    def should_ignore(self, path: Path) -> bool:
        """Check if a file or directory should be ignored"""
        path_str = str(path)

        # Check exact matches
        for pattern in self.ignore_patterns:
            if pattern in path_str:
                return True

        # Check file extensions
        if path.is_file():
            suffix = path.suffix.lower()
            if suffix in [".pyc", ".pyo", ".pyd", ".so", ".dll", ".exe", ".app", ".dmg", ".pkg"]:
                return True

        return False

    def remove_emojis(self, text: str) -> tuple[str, int]:
        """Remove emojis from text and return cleaned text and count"""
        cleaned_text = self.emoji_pattern.sub("", text)
        removed_count = len(text) - len(cleaned_text)
        return cleaned_text, removed_count

    def process_file(self, file_path: Path) -> bool:
        """Process a single file and return True if modified"""
        try:
            # Read file content
            with open(file_path, encoding="utf-8") as file:
                content = file.read()

            # Remove emojis
            cleaned_content, emoji_count = self.remove_emojis(content)

            if emoji_count > 0:
                # Write back cleaned content
                with open(file_path, "w", encoding="utf-8") as file:
                    file.write(cleaned_content)

                self.stats["emojis_removed"] += emoji_count
                self.stats["files_modified"] += 1
                return True
            else:
                self.stats["files_skipped"] += 1
                return False

        except Exception as e:
            print(f"\nError processing {file_path}: {e}")
            self.stats["errors"] += 1
            return False

    def get_text_files(self, directory: Path) -> list[Path]:
        """Get all text files in directory recursively"""
        text_extensions = {
            ".txt",
            ".md",
            ".rst",
            ".adoc",
            ".tex",
            ".html",
            ".htm",
            ".xml",
            ".json",
            ".yaml",
            ".yml",
            ".toml",
            ".ini",
            ".cfg",
            ".conf",
            ".py",
            ".js",
            ".ts",
            ".jsx",
            ".tsx",
            ".vue",
            ".php",
            ".rb",
            ".java",
            ".c",
            ".cpp",
            ".h",
            ".hpp",
            ".cs",
            ".go",
            ".rs",
            ".swift",
            ".kt",
            ".scala",
            ".clj",
            ".hs",
            ".ml",
            ".fs",
            ".sql",
            ".sh",
            ".bash",
            ".zsh",
            ".fish",
            ".ps1",
            ".bat",
            ".css",
            ".scss",
            ".sass",
            ".less",
            ".styl",
            ".dockerfile",
            ".gitignore",
            ".gitattributes",
            "readme",
            "license",
            "changelog",
            "contributing",
        }

        text_files = []

        for file_path in directory.rglob("*"):
            if file_path.is_file() and not self.should_ignore(file_path):
                # Check if it's a text file
                if file_path.suffix.lower() in text_extensions or file_path.name.lower() in text_extensions:
                    text_files.append(file_path)

        return text_files

    def process_directory(self, directory_path: Path) -> None:
        """Process all text files in directory recursively"""
        print(f"\nScanning directory: {directory_path}")

        # Get all text files
        text_files = self.get_text_files(directory_path)

        if not text_files:
            self.print_border("No text files found to process!", "info")
            return

        print(f"\nFound {len(text_files)} text files to process")

        # Confirmation
        confirm_text = f"""
        Process {len(text_files)} files in {directory_path}?

        This will recursively scan and remove emojis from all text files.
        Ignored patterns: .venv, __pycache__, .git, node_modules, etc.

        Proceed? (y/n):
        """
        self.print_border(confirm_text, "info")

        choice = input("> ").lower().strip()
        if choice != "y":
            self.print_border("Operation cancelled. No changes made.", "info")
            return

        # Process files with progress bar
        print("\nProcessing files...")
        for i, file_path in enumerate(text_files, 1):
            self.print_progress_bar(i, len(text_files), f"Processing {file_path.name}")

            if self.process_file(file_path):
                print(f"\nModified: {file_path}")

            self.stats["files_processed"] += 1

    def show_welcome(self) -> None:
        """Display the welcome screen"""
        welcome_text = """
        
         > nutshell.os                                                                
           .-.                                                                        
          {_}                                                                       
          { . }   Main Menu                                                           
          {___}                                                                       
                                                                                      
         > ...emoji pulverizer5000                                                    
         > ...boot: Success!                                                          
                                                                                      
        Brought to you by:
        TheBurningPeanut and TheBungilators
        
        """
        print(welcome_text)

    def show_menu(self) -> None:
        """Display the main menu"""
        menu_text = """


        
         > nutshell.os                                                                
           .-.                                                                        
          {_}                                                                       
          { . }   Main Menu                                                           
          {___}                                                                       
                                                                                      
          1. Process a single file                                                    
          2. Process a directory (recursive)                                          
          3. Show statistics                                                          
          4. Exit                                                                     
                                                                                      
        Brought to you by:
        TheBurningPeanut and TheBungilators
        
        """
        print(menu_text)

    def process_single_file(self) -> None:
        """Process a single file"""
        print("\nSingle File Processing")
        print("=" * 50)

        file_path = input("Enter the path to the file: ").strip()

        if not file_path:
            self.print_border("No file path provided!", "error")
            return

        path = Path(file_path)

        if not path.exists():
            self.print_border(f"File not found: {file_path}", "error")
            return

        if not path.is_file():
            self.print_border(f"Path is not a file: {file_path}", "error")
            return

        if self.should_ignore(path):
            self.print_border(f"File is in ignore list: {file_path}", "error")
            return

        # Process the file
        print(f"\nProcessing: {path}")

        if self.process_file(path):
            self.print_border(f"Successfully removed emojis from {path.name}!", "success")
        else:
            self.print_border(f"No emojis found in {path.name}", "info")

    def show_statistics(self) -> None:
        """Display processing statistics"""
        files_processed = self.stats["files_processed"]
        files_modified = self.stats["files_modified"]
        files_skipped = self.stats["files_skipped"]
        emojis_removed = self.stats["emojis_removed"]
        errors = self.stats["errors"]

        stats_text = f"""
emoji Destroyer Stats 5000  - nutshell.os
-----------------------------------------
Files Processed: {files_processed:>8}
Files Modified:  {files_modified:>8}
Files Skipped:   {files_skipped:>8}
Emojis Removed:  {emojis_removed:>8}
Errors:          {errors:>8}

                 
            Brought to you by                
         TheBurningPeanut and TheBungilators      
                 
        """
        self.print_border(stats_text, "info")

    def run(self) -> None:
        """Main application loop"""
        self.show_welcome()

        while True:
            self.show_menu()
            choice = input("Select an option (1-4): ").strip()

            if choice == "1":
                self.process_single_file()
            elif choice == "2":
                print("\nDirectory Processing")
                print("=" * 50)
                dir_path = input("Enter the directory path: ").strip()

                if not dir_path:
                    self.print_border("No directory path provided!", "error")
                    continue

                path = Path(dir_path)
                if not path.exists():
                    self.print_border(f"Directory not found: {dir_path}", "error")
                    continue

                if not path.is_dir():
                    self.print_border(f"Path is not a directory: {dir_path}", "error")
                    continue

                self.process_directory(path)

            elif choice == "3":
                self.show_statistics()
            elif choice == "4":
                self.print_border("Thanks for using Emoji Scrubber!", "success")
                break
            else:
                self.print_border("Invalid option! Please select 1-4.", "error")

            input("\nPress Enter to continue...")


def main() -> None:
    """Main entry point"""
    try:
        scrubber = EmojiScrubber()
        scrubber.run()
    except KeyboardInterrupt:
        print("\n\nOperation cancelled by user.")
        sys.exit(0)
    except Exception as e:
        print(f"\nUnexpected error: {e}")
        sys.exit(1)


if __name__ == "__main__":
    main()

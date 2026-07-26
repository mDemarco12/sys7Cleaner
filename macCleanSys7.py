
import tkinter as tk
from tkinter import ttk, filedialog, messagebox
from pathlib import Path
import os

BG = "#d8d8d8"

class System7Cleaner:
    def __init__(self, root):
        self.root = root
        root.title("Macintosh HD Cleanup Utility")
        root.geometry("900x650")
        root.configure(bg=BG)

        title = tk.Label(root, text="Macintosh HD Cleanup Utility",
                         bg=BG, font=("Geneva", 18, "bold"))
        title.pack(pady=8)

        toolbar = tk.Frame(root, bg=BG, relief="raised", bd=2)
        toolbar.pack(fill="x", padx=8)

        tk.Button(toolbar, text="Scan Mac", command=self.scan).pack(side="left", padx=3, pady=3)
        tk.Button(toolbar, text="Large Files", command=self.large_files).pack(side="left", padx=3, pady=3)
        tk.Button(toolbar, text="Intel Apps", command=self.intel_apps).pack(side="left", padx=3, pady=3)
        tk.Button(toolbar, text="Clear Caches", command=self.clear_caches).pack(side="left", padx=3, pady=3)

        self.text = tk.Text(root, bg="white", relief="sunken")
        self.text.pack(fill="both", expand=True, padx=8, pady=8)

        self.log("Welcome to the Macintosh HD Cleanup Utility")
        self.log("Designed for Apple Silicon migration cleanup.\n")

    def log(self, msg):
        self.text.insert("end", msg + "\n")
        self.text.see("end")

    def size(self, p):
        if not p.exists():
            return 0
        total = 0
        for root, _, files in os.walk(p):
            for f in files:
                try:
                    total += os.path.getsize(os.path.join(root, f))
                except:
                    pass
        return total

    def human(self, n):
        for u in ["B","KB","MB","GB","TB"]:
            if n < 1024:
                return f"{n:.1f} {u}"
            n /= 1024
        return f"{n:.1f} PB"

    def scan(self):
        self.text.delete("1.0","end")
        home = Path.home()

        locations = {
            "Caches": home/"Library/Caches",
            "Downloads": home/"Downloads",
            "iOS Backups": home/"Library/Application Support/MobileSync/Backup",
            "Xcode DerivedData": home/"Library/Developer/Xcode/DerivedData",
            "Xcode Archives": home/"Library/Developer/Xcode/Archives",
            "Docker": home/"Library/Containers/com.docker.docker"
        }

        total = 0
        self.log("=== Storage Analysis ===\n")

        for name, path in locations.items():
            s = self.size(path)
            total += s
            self.log(f"{name:20} {self.human(s)}")

        self.log("\nPotential cleanup total: " + self.human(total))

    def large_files(self):
        self.text.delete("1.0","end")
        self.log("Scanning for files larger than 500 MB...\n")

        results = []
        for root, _, files in os.walk(Path.home()):
            for f in files:
                try:
                    p = Path(root)/f
                    sz = p.stat().st_size
                    if sz > 500 * 1024 * 1024:
                        results.append((sz, str(p)))
                except:
                    pass

        for sz, p in sorted(results, reverse=True)[:100]:
            self.log(f"{self.human(sz):>10}  {p}")

        if not results:
            self.log("No files over 500 MB found.")

    def intel_apps(self):
        self.text.delete("1.0","end")
        self.log("Checking Applications folder...\n")
        apps = Path("/Applications")

        found = False
        for item in sorted(apps.glob("*.app")):
            self.log(item.name)
            found = True

        if not found:
            self.log("No applications found.")

        self.log("\nTip: In Finder add the 'Kind' column or use Activity Monitor > Architecture to identify Intel-only apps.")

    def clear_caches(self):
        cache = Path.home()/"Library/Caches"
        if not messagebox.askyesno("Confirm", "Delete contents of ~/Library/Caches ?"):
            return

        removed = 0
        for item in cache.iterdir():
            try:
                if item.is_dir():
                    os.system(f'rm -rf "{item}"')
                else:
                    item.unlink()
                removed += 1
            except:
                pass

        messagebox.showinfo("Finished", f"Removed {removed} cache entries.")

root = tk.Tk()
System7Cleaner(root)
root.mainloop()

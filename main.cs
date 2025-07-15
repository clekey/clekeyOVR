using System;
using System.Runtime.InteropServices;


Console.WriteLine(GetProcessLockingClipboard().Id);


[DllImport("user32.dll", SetLastError = true)]
static extern IntPtr GetOpenClipboardWindow();

[DllImport("user32.dll", SetLastError = true)]
static extern IntPtr GetClipboardOwner();

[DllImport("user32.dll", SetLastError = true)]
static extern int GetWindowThreadProcessId(IntPtr hWnd, out int lpdwProcessId);

private static Process GetProcessLockingClipboard()
{
IntPtr handle = GetOpenClipboardWindow();
int error = Marshal.GetLastWin32Error();

Console.WriteLine(handle);
Console.WriteLine(error);

    int processId;
    GetWindowThreadProcessId(handle, out processId);

    return Process.GetProcessById(processId);
}

/**
 * Counts lines in a Google Doc that contain specific symbols,
 * associates them with defined "states," and reports the counts
 * at the top of the document. This version replaces any existing
 * report at the top of the document.
 *
 * It also includes new logic to modify document content before reporting:
 * For each Heading 3 (H3) containing "🔶":
 * - It then iterates through subsequent normal lines in that section:
 * - If the FIRST line has "🔵", it does nothing and stops processing that H3 section.
 * - If a line has "🔴", it changes "🔴" to "🔵" and stops processing that H3 section.
 * - If a line has "🔶", it does nothing and stops processing that H3 section.
 * - If a line has "✅", it does nothing and continues to the next line in that H3 section.
 *
 * IMPORTANT COUNTING LOGIC:
 * - Only symbols on "normal" lines (Normal Text, List Item, Unordered List Item)
 * that are DIRECTLY within a Heading 3 section containing "🔶" are counted.
 * Symbols outside of these sections (e.g., in H1 "Done" section) are IGNORED for counting.
 *
 * NEW FEATURE:
 * - The report now includes clickable links for "On Deck" (🔵) and "In Progress" (🔶) items,
 * allowing you to jump directly to those lines in the document.
 *
 * This version includes Logger.log() statements for debugging.
 *
 * To use:
 * 1. Open your Google Doc.
 * 2. Go to Extensions > Apps Script.
 * 3. Delete any existing code and paste this code into the editor.
 * 4. Save the project (File > Save project or Ctrl+S/Cmd+S).
 * 5. In the Apps Script editor, select the `countAndReportSymbols` function from the dropdown.
 * 6. Click the "Run" button (play icon) or use the custom menu in your Google Doc.
 * 7. You may be prompted to authorize the script; follow the steps to grant privacy permissions.
 * 8. Once executed, the report will appear at the top of your Google Doc,
 * replacing any previous report, and specific lines will be modified.
 * 9. To view the debug logs, in the Apps Script editor, go to the "Executions" tab (left sidebar, clock icon).
 * Click on the most recent execution to see the detailed logs.
 */

/**
 * Runs automatically when the Google Doc is opened.
 * Creates a custom menu to easily refresh the symbol report and archive done tasks.
 */
function onOpen() {
  DocumentApp.getUi()
    .createMenu('GTD Report') // Name of your custom menu
    .addItem('Refresh Symbol Report', 'countAndReportSymbols') // Menu item and function to call
    .addItem('Archive Done Tasks', 'archiveDoneTasks') // New menu item
    .addToUi();
}




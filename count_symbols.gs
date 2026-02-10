/**
 * Main function to count symbols and generate the report.
 * Does NOT call doc.saveAndClose() itself, allowing calling functions
 * to control when the document is saved and closed.
 */
function countAndReportSymbols() {
  const constants = getDocConstants();
  const { doc, body } = initializeDocument();

  if (!doc) {
    return; // Document was empty, message already inserted.
  }

  // 1. Process document for symbol modifications
  processDocumentSymbols(body, constants);

  // 2. Delete existing report (important to do AFTER modifications to avoid deleting newly modified lines)
  // Note: updatedParagraphsForDeletion is no longer strictly needed here,
  // as deleteExistingReport will fetch paragraphs directly.
  deleteExistingReport(body, constants.REPORT_START_MARKER, constants.REPORT_END_MARKER);

  // 3. Clear all existing bookmarks
  clearAllExistingBookmarks(doc);

  // 4. Count symbols and create new bookmarks
  const { counts, bookmarkedOnDeckLines, bookmarkedInProgressLines } = countSymbolsAndCreateBookmarks(doc, body, constants);

  // 5. Generate report content
  const reportContent = generateReportContent(counts, bookmarkedOnDeckLines, bookmarkedInProgressLines, constants);

  // 6. Insert the report into the document
  insertReportIntoDocument(doc, body, reportContent);

  Logger.log("countAndReportSymbols execution complete.");
}

// --- Independent Helper Functions ---

/**
 * Defines and returns an object containing all document-wide constants,
 * including Unicode symbols and their corresponding states.
 * This centralizes configuration and improves maintainability.
 */
function getDocConstants() {
  const SYMBOLS = {
    TODO: "🔴",
    ON_DECK: "🔵",
    IN_PROGRESS: "🔶",
    DONE: "✅",
  };

  return {
    REPORT_START_MARKER: "Agenda",
    REPORT_SUBHEADING: "―".repeat(40),
    REPORT_END_MARKER: "―".repeat(41),
    LINE_SYMBOL_DEBUG: false, // Flag for internal debugging logs
    SHOW_COUNT_REPORT: false, // Changed to false by default
    SYMBOLS: SYMBOLS, // Expose the symbols object directly

    // Derive SYMBOL_STATES from SYMBOLS for consistent naming
    SYMBOL_STATES: {
      [SYMBOLS.TODO]: "Todo",
      [SYMBOLS.ON_DECK]: "On Deck",
      [SYMBOLS.IN_PROGRESS]: "In Progress",
      [SYMBOLS.DONE]: "Done",
    },
  };
}

/**
 * Initializes the document and performs an empty document check.
 * If the document is empty, it inserts a message and returns null.
 *
 * @returns {object|null} An object containing the Google Document `doc` and `body`
 * if the document is valid, otherwise null.
 */
function initializeDocument() {
  const doc = DocumentApp.getActiveDocument();
  const body = doc.getBody();

  // Check if the document is empty before attempting any operations
  if (body.getNumChildren() === 0 || (body.getNumChildren() === 1 && body.getChild(0).asParagraph().getText().trim() === '')) {
    body.insertParagraph(0, "Document is empty. No content to analyze or modify.");
    return { doc: null, body: null }; // Indicate that the document is not ready
  }
  return { doc, body };
}

/**
 * Processes the document body to apply symbol modification logic.
 * Specifically, it replaces 'TODO' symbols (🔴) with 'ON_DECK' symbols (🔵)
 * within sections marked by an 'IN_PROGRESS' heading (🔶).
 *
 * @param {GoogleAppsScript.Document.Body} body The body of the active Google Document.
 * @param {object} constants An object containing all predefined constants, including symbols.
 */
function processDocumentSymbols(body, constants) {
  Logger.log("--- Starting Document Modification ---");
  let inTargetH3Section = false; // Flag to indicate if we are currently inside an H3 section triggered by "🔶"
  let currentChildIndex = 0;

  // Use a while loop to handle dynamic child removal or addition without breaking iteration
  while (currentChildIndex < body.getNumChildren()) {
    const child = body.getChild(currentChildIndex);

    if (constants.LINE_SYMBOL_DEBUG) Logger.log(`Processing child ${currentChildIndex + 1}/${body.getNumChildren()}. Type: ${child.getType()}`);

    // Process Paragraph elements for heading and text checks
    if (child.getType() === DocumentApp.ElementType.PARAGRAPH) {
      const paragraph = child.asParagraph();
      const heading = paragraph.getHeading();
      const text = paragraph.getText();

      if (constants.LINE_SYMBOL_DEBUG) Logger.log(`  Paragraph Text: "${text.trim()}", Heading Type: ${heading}`);

      // --- PRIMARY LOGIC BRANCHING ---

      // 1. If we are currently IN a target H3 section AND this is a "normal" line type (NORMAL/LIST_ITEM).
      // This branch is prioritized to process lines *within* an active section.
      if (inTargetH3Section && (heading === DocumentApp.ParagraphHeading.NORMAL ||
          heading === DocumentApp.ParagraphHeading.LIST_ITEM ||
          heading === DocumentApp.ParagraphHeading.UNORDERED_LIST_ITEM)) {

        if (constants.LINE_SYMBOL_DEBUG) Logger.log(`Inside ACTIVE TARGET H3 section. Current inTargetH3Section: ${inTargetH3Section}. Checking normal line: "${text.trim()}"`);

        // LOGIC: If the line contains "ON_DECK" (🔵), just break this H3 section's processing.
        if (text.includes(constants.SYMBOLS.ON_DECK)) {
          if (constants.LINE_SYMBOL_DEBUG) Logger.log(`Action: Found "${constants.SYMBOLS.ON_DECK}" on line. Breaking. Deactivating inTargetH3Section.`);
          inTargetH3Section = false; // Stop processing this H3 section

        } else if (text.includes(constants.SYMBOLS.TODO)) {
          // Replace all occurrences of 'TODO' (🔴) with 'ON_DECK' (🔵) on the line
          // Use RegExp with 'g' flag for global replacement
          paragraph.setText(text.replace(new RegExp(constants.SYMBOLS.TODO, 'g'), constants.SYMBOLS.ON_DECK));
          if (constants.LINE_SYMBOL_DEBUG) Logger.log(`Action: Replaced "${constants.SYMBOLS.TODO}" with "${constants.SYMBOLS.ON_DECK}". New text: "${paragraph.getText().trim()}". Deactivating inTargetH3Section.`);
          inTargetH3Section = false; // Stop processing this H3 section after modification

        } else if (text.includes(constants.SYMBOLS.IN_PROGRESS)) {
          if (constants.LINE_SYMBOL_DEBUG) Logger.log(`Action: Found "${constants.SYMBOLS.IN_PROGRESS}". Do nothing. Deactivating inTargetH3Section.`);
          inTargetH3Section = false; // Do nothing as per requirement, and stop processing this H3 section

        } else if (text.includes(constants.SYMBOLS.DONE)) {
          if (constants.LINE_SYMBOL_DEBUG) Logger.log(`Action: Found "${constants.SYMBOLS.DONE}". Do nothing and continue to the next line in section.`);
          // inTargetH3Section remains true here as per requirement

        } else {
          // This is a normal line, but without 🔴, 🔶, or ✅.
          // We stay within the section to continue checking subsequent lines.
          if (constants.LINE_SYMBOL_DEBUG) Logger.log(`No target symbol found on line. Continuing within section. inTargetH3Section remains ${inTargetH3Section}.`);
        }
      }
      // 2. If it's a Heading 3. This branch processes H3s to activate/deactivate the flag.
      else if (heading === DocumentApp.ParagraphHeading.HEADING3) {

        if (text.includes(constants.SYMBOLS.IN_PROGRESS)) {
          if (constants.LINE_SYMBOL_DEBUG) Logger.log(`Found TARGET H3 with "${constants.SYMBOLS.IN_PROGRESS}": "${text.trim()}". Activating inTargetH3Section.`);
          inTargetH3Section = true; // Activate for this new section

        } else {
          // It's an H3, but not the target 🔶 H3. This ends any previous target section.
          if (inTargetH3Section) {
            Logger.log(`Found non-target H3: "${text.trim()}". Deactivating inTargetH3Section.`);
          }
          inTargetH3Section = false;
        }
      }
      // 3. If it's any other section-breaking heading (H1, H2, H4-H6, Title, Subtitle).
      // This branch always signifies the end of any active target H3 section.
      else if (
        heading === DocumentApp.ParagraphHeading.HEADING1 ||
        heading === DocumentApp.ParagraphHeading.HEADING2 ||
        heading === DocumentApp.ParagraphHeading.HEADING4 ||
        heading === DocumentApp.ParagraphHeading.HEADING5 ||
        heading === DocumentApp.ParagraphHeading.HEADING6 ||
        heading === DocumentApp.ParagraphHeading.TITLE ||
        heading === DocumentApp.ParagraphHeading.SUBTITLE
      ) {
        if (inTargetH3Section) {
          Logger.log(`Found higher/different level heading (${heading}): "${text.trim()}". Deactivating inTargetH3Section.`);
        }
        inTargetH3Section = false;
      }
      // 4. Fallback for any other PARAGRAPH types not explicitly handled above.
      // If inTargetH3Section is true here, it means an unhandled paragraph type occurred,
      // implicitly breaking the section flow.
      else if (inTargetH3Section) {
        Logger.log(`Found an unhandled PARAGRAPH type (${heading}) while in a target H3 section. Deactivating.`);
        inTargetH3Section = false;
      }
      // 5. If none of the above conditions, and not in an H3 section, it's just a regular paragraph
      // outside any special scope. No change to inTargetH3Section.
      else {
        if (constants.LINE_SYMBOL_DEBUG) Logger.log(`Not in target H3 section, and not a section-breaking heading. No change to inTargetH3Section.`);
      }
    }
    // Handle non-PARAGRAPH elements (e.g., tables, horizontal rules, images).
    // These always signify the end of any active text-based H3 section.
    else {
      if (inTargetH3Section) {
        Logger.log(`Found non-paragraph element (${child.getType()}). Deactivating inTargetH3Section.`);
      }
      inTargetH3Section = false;
    }
    currentChildIndex++; // Always move to the next child in the document
  }
  Logger.log("--- Finished Document Modification ---");
}

/**
 * Clears all existing bookmarks from the given Google Document.
 *
 * @param {GoogleAppsScript.Document.Document} doc The active Google Document.
 */
function clearAllExistingBookmarks(doc) {
  Logger.log("--- Clearing existing bookmarks ---");
  const existingBookmarks = doc.getBookmarks();
  if (existingBookmarks) Logger.log(`Found ${existingBookmarks.length} existing bookmarks.`);

  for (let i = existingBookmarks.length - 1; i >= 0; i--) {
    const bookmark = existingBookmarks[i];
    if (Logger) Logger.log(`  Removing bookmark: ${bookmark.getId()}`); // Conditional logging
    bookmark.remove(); // see https://developers.google.com/apps-script/reference/document/bookmark
  }
  Logger.log("--- Finished clearing existing bookmarks ---");
}

/**
 * Deletes any existing report section from the document body, identified by
 * start and end markers.
 *
 * @param {GoogleAppsScript.Document.Body} body The body of the active Google Document.
 * @param {string} reportStartMarker The string that marks the beginning of the report.
 * @param {string} reportEndMarker The string that marks the end of the report.
 */
function deleteExistingReport(body, reportStartMarker, reportEndMarker) {
  Logger.log("--- Deleting existing report ---");
  const paragraphs = body.getParagraphs(); // Get a fresh list of paragraphs
  let reportStartIndex = -1;
  let reportEndIndex = -1;

  for (let i = 0; i < paragraphs.length; i++) {
    const paragraphText = paragraphs[i].getText().trim();
    if (reportStartIndex === -1 && paragraphText === reportStartMarker) {
      reportStartIndex = i;
    }
    if (reportStartIndex !== -1 && paragraphText === reportEndMarker) {
      reportEndIndex = i;
      break;
    }
  }

  if (reportStartIndex !== -1 && reportEndIndex !== -1) {
    // Iterate in reverse to safely remove children without affecting indices for subsequent removals
    for (let i = reportEndIndex; i >= reportStartIndex; i--) {
      body.getChild(i).removeFromParent();
    }
    Logger.log(`Deleted report from index ${reportStartIndex} to ${reportEndIndex}.`);
  } else {
    Logger.log("No existing report found to delete.");
  }
  Logger.log("--- Finished deleting existing report ---");
}

/**
 * Counts the occurrences of defined symbols within the document and creates bookmarks
 * for lines containing "On Deck" (🔵) or "In Progress" (🔶) symbols.
 *
 * @param {GoogleAppsScript.Document.Document} doc The active Google Document.
 * @param {GoogleAppsScript.Document.Body} body The body of the active Google Document.
 * @param {object} constants An object containing all predefined constants, including symbols.
 * @returns {object} An object containing `counts` of each symbol state,
 * `bookmarkedOnDeckLines`, and `bookmarkedInProgressLines`.
 */
function countSymbolsAndCreateBookmarks(doc, body, constants) {
  Logger.log("--- Starting Counting and Bookmarking ---");
  // Initialize counts for each defined state to zero.
  const counts = {};
  for (const symbolStateName in constants.SYMBOL_STATES) { // Iterate using the keys of SYMBOL_STATES (the actual unicode symbols)
    counts[constants.SYMBOL_STATES[symbolStateName]] = 0; // Initialize with the friendly name as key
  }

  // Arrays to store bookmarked lines for the report
  const bookmarkedOnDeckLines = [];
  const bookmarkedInProgressLines = [];

  // Reset the flag for counting as we're doing a fresh pass
  let inTargetH3SectionForCounting = false;

  const paragraphs = body.getParagraphs(); // Get the current state of paragraphs

  paragraphs.forEach(paragraph => {
    const lineText = paragraph.getText();
    const heading = paragraph.getHeading();

    // Replicate the section detection logic for counting purposes
    if (heading === DocumentApp.ParagraphHeading.HEADING3) {
      inTargetH3SectionForCounting = lineText.includes(constants.SYMBOLS.IN_PROGRESS);
    } else if (
      // Any other heading (H1, H2, H4-H6, Title, Subtitle) breaks the section for counting
      heading === DocumentApp.ParagraphHeading.HEADING1 ||
      heading === DocumentApp.ParagraphHeading.HEADING2 ||
      heading === DocumentApp.ParagraphHeading.HEADING4 ||
      heading === DocumentApp.ParagraphHeading.HEADING5 ||
      heading === DocumentApp.ParagraphHeading.HEADING6 ||
      heading === DocumentApp.ParagraphHeading.TITLE ||
      heading === DocumentApp.ParagraphHeading.SUBTITLE ||
      paragraph.getType() !== DocumentApp.ElementType.PARAGRAPH // Non-paragraph elements also break section for text analysis
    ) {
      inTargetH3SectionForCounting = false;
    }

    // ONLY count if we are in a target H3 section AND it's a "normal" line type
    if (inTargetH3SectionForCounting && (heading === DocumentApp.ParagraphHeading.NORMAL ||
        heading === DocumentApp.ParagraphHeading.LIST_ITEM ||
        heading === DocumentApp.ParagraphHeading.UNORDERED_LIST_ITEM)) {

      if (constants.LINE_SYMBOL_DEBUG) Logger.log(`Counting line within ACTIVE TARGET H3 section: "${lineText.trim()}"`);

      // Iterate through the actual symbol characters to check for inclusion
      for (const symbolKey in constants.SYMBOLS) {
        const symbol = constants.SYMBOLS[symbolKey]; // Get the actual Unicode character
        if (lineText.includes(symbol)) {
          // Increment count using the friendly name from SYMBOL_STATES
          counts[constants.SYMBOL_STATES[symbol]]++;
          // This is the CORRECT bookmark creation logic for the line being analyzed:
          if (symbol === constants.SYMBOLS.ON_DECK || symbol === constants.SYMBOLS.IN_PROGRESS) {
            try {
              const startPosition = getParagraphStartPosition(paragraph);
              const bookmark = doc.addBookmark(startPosition);
              const actualBookmarkId = bookmark.getId();
              // Remove the IN_PROGRESS symbol from the parent text for cleaner display
              const parentText = getNearestH3ParentText(paragraph).replace(new RegExp(constants.SYMBOLS.IN_PROGRESS, 'g'), "").trim();
              const linkText = `${lineText.trim()} (${parentText})`;
              Logger.log(`    Added bookmark "${actualBookmarkId}" to line: "${linkText}"`);
              if (symbol === constants.SYMBOLS.ON_DECK) {
                bookmarkedOnDeckLines.push({ text: linkText, bookmarkId: actualBookmarkId });
              } else if (symbol === constants.SYMBOLS.IN_PROGRESS) {
                bookmarkedInProgressLines.push({ text: linkText, bookmarkId: actualBookmarkId });
              }
            } catch (e) {
              Logger.log(`    ERROR creating bookmark for line "${lineText.trim()}": ${e.message}`);
            }
          }
        }
      }
    } else {
      if (constants.LINE_SYMBOL_DEBUG) Logger.log(`Skipping line for counting (not in target H3 section or not normal line): "${lineText.trim()}"`);
    }
  });
  Logger.log("--- Finished Counting and Bookmarking ---");
  return { counts, bookmarkedOnDeckLines, bookmarkedInProgressLines };
}

/**
 * Generates the content for the report as an array of structured items
 * (text lines or link objects).
 *
 * @param {object} counts An object containing the counts of each symbol state.
 * @param {Array<object>} bookmarkedOnDeckLines An array of objects for 'On Deck' lines with bookmark info.
 * @param {Array<object>} bookmarkedInProgressLines An array of objects for 'In Progress' lines with bookmark info.
 * @param {object} constants An object containing all predefined constants.
 * @returns {Array<object>} An array of report items (text or link objects).
 */
function generateReportContent(counts, bookmarkedOnDeckLines, bookmarkedInProgressLines, constants) {
  Logger.log("--- Generating Report Content ---");
  let finalReportItems = [];
  let hasResults = false;

  finalReportItems.push({ type: 'header', content: constants.REPORT_START_MARKER });
  finalReportItems.push({ type: 'text', content: constants.REPORT_SUBHEADING });

  if (constants.SHOW_COUNT_REPORT) {
    finalReportItems.push({ type: 'text', content: `Generated: ${new Date().toLocaleString()}` });
    for (const state in counts) {
      if (counts[state] > 0) {
        finalReportItems.push({ type: 'text', content: `${state}: ${counts[state]}` });
        hasResults = true;
      }
    }
  }

  // Add detailed In Progress items if any (🔶) - first as requested
  if (bookmarkedInProgressLines.length > 0) {
    finalReportItems.push({ type: 'text', content: "" }); // Separator
    // finalReportItems.push({ type: 'text', content: `${constants.SYMBOLS.IN_PROGRESS} In Progress` }); // Updated heading for clarity
    bookmarkedInProgressLines.forEach(item => {
      finalReportItems.push({ type: 'link', content: `${item.text}`, bookmarkId: item.bookmarkId });
    });
    hasResults = true;
  }

  // Add detailed On Deck items if any (🔵) - second as requested
  if (bookmarkedOnDeckLines.length > 0) {
    finalReportItems.push({ type: 'text', content: "" }); // Separator
    // finalReportItems.push({ type: 'text', content: `${constants.SYMBOLS.ON_DECK} On Deck` }); // Updated heading for clarity
    bookmarkedOnDeckLines.forEach(item => {
      finalReportItems.push({ type: 'link', content: `${item.text}`, bookmarkId: item.bookmarkId });
    });
    hasResults = true;
  }

  if (!hasResults) {
    finalReportItems.push({ type: 'text', content: "No lines found containing the configured symbols." });
  }
  finalReportItems.push({ type: 'text', content: "" });
  finalReportItems.push({ type: 'text', content: constants.REPORT_END_MARKER });
  Logger.log("--- Finished Generating Report Content ---");
  return finalReportItems;
}


/**
 * Inserts the generated report content into the document body at the very top.
 * Handles both plain text lines and bookmarked links. This version
 * applies separate colors to text enclosed in parentheses, dates (e.g., ..9/2),
 * and plus-number patterns (e.g., +3).
 *
 * @param {GoogleAppsScript.Document.Document} doc The active Google Document.
 * @param {GoogleAppsScript.Document.Body} body The body of the active Google Document.
 * @param {Array<object>} reportItems An array of report items (text or link objects) to insert.
 */
function insertReportIntoDocument(doc, body, reportItems) {
  Logger.log("--- Inserting Report into Document ---");
  const docId = doc.getId();

  const parensColor = "#F4B400";     // Google's yellow for (parens)
  const dateColor = "#DB4437";       // Google's red for dates (e.g., ..9/2)
  const plusNumColor = "#4285F4";    // Google's blue for plus-numbers (e.g., +3)
  const defaultColor = "#FFF2CC";    // A light default color

  const parensRegex = /\s*\([^)]*\)$/g;
  const dateRegex = /\.\.\d{1,2}\/\d{1,2}/g;
  const plusNumRegex = /\+\d+/g;

  for (let i = reportItems.length - 1; i >= 0; i--) {
    const reportItem = reportItems[i];
    
    // Process both text and link types similarly for formatting
    let paragraph;
    if (reportItem.type === 'text') {
      paragraph = body.insertParagraph(0, reportItem.content);
    } else if (reportItem.type === 'link') {
      paragraph = body.insertParagraph(0, reportItem.content);
      const linkUrl = `https://docs.google.com/document/d/${docId}/edit#bookmark=${reportItem.bookmarkId}`;
      paragraph.editAsText().setLinkUrl(linkUrl);
    } else if (reportItem.type === 'header') {
      paragraph = body.insertParagraph(0, reportItem.content);
      paragraph.setHeading(DocumentApp.ParagraphHeading.HEADING2);
      continue; // Skip formatting for headers
    } else {
      continue; // Skip any other unhandled types
    }

    const textElement = paragraph.editAsText();
    const fullText = reportItem.content;

    // Apply default color to the entire line first
    textElement.setForegroundColor(defaultColor);
    if (reportItem.type === 'link') {
      textElement.setUnderline(false); // Remove underline for links
    }

    // Use a single array to hold all matches and their colors
    const matches = [];
    
    // Find all matches for the plus-number pattern
    const plusNumMatches = fullText.matchAll(plusNumRegex);
    for (const match of plusNumMatches) {
      matches.push({ start: match.index, end: match.index + match[0].length, color: plusNumColor });
    }

    // Find all matches for the date pattern
    const dateMatches = fullText.matchAll(dateRegex);
    for (const match of dateMatches) {
      matches.push({ start: match.index, end: match.index + match[0].length, color: dateColor });
    }

    // Find all matches for the parentheses pattern
    const parensMatches = fullText.matchAll(parensRegex);
    for (const match of parensMatches) {
      matches.push({ start: match.index, end: match.index + match[0].length, color: parensColor });
    }
    
    // Sort matches by their start index to avoid conflicts
    matches.sort((a, b) => a.start - b.start);

    // Apply colors to the matched ranges in the correct order
    matches.forEach(m => {
      textElement.setForegroundColor(m.start, m.end - 1, m.color);
    });
  }
  Logger.log("--- Finished Inserting Report ---");
}


/**
 * Traverses backward from a given paragraph to find the text content of the nearest
 * preceding H3 heading.
 *
 * @param {GoogleAppsScript.Document.Paragraph} childParagraph The paragraph from which to start searching backward.
 * @returns {string} The text of the nearest H3 parent, or an empty string if none is found
 * or a higher-level heading/non-paragraph element is encountered first.
 * @throws {Error} If the input is not a Paragraph object.
 */
function getNearestH3ParentText(childParagraph) {
  if (!childParagraph || childParagraph.getType() !== DocumentApp.ElementType.PARAGRAPH) {
    throw new Error("Invalid input: Expected a Paragraph object.");
  }

  const body = DocumentApp.getActiveDocument().getBody();
  const childIndex = body.getChildIndex(childParagraph); // Get the index of the current paragraph

  // Traverse backward from the current paragraph's index
  for (let i = childIndex - 1; i >= 0; i--) {
    const precedingElement = body.getChild(i);

    // Only process Paragraph elements
    if (precedingElement.getType() === DocumentApp.ElementType.PARAGRAPH) {
      const precedingParagraph = precedingElement.asParagraph();
      const heading = precedingParagraph.getHeading();

      // If we find an H3, that's our target. Return its text.
      if (heading === DocumentApp.ParagraphHeading.HEADING3) {
        return precedingParagraph.getText().trim();
      }

      // If we encounter a heading of H1 or H2, it means we've passed
      // a major section boundary, so the H3 we're looking for (if any)
      // would not be a "parent" to this paragraph in the desired logical sense.
      // Stop searching this chain.
      if (heading === DocumentApp.ParagraphHeading.HEADING1 ||
          heading === DocumentApp.ParagraphHeading.HEADING2) {
        return ""; // No relevant H3 found in this section
      }
    } else {
      // If a non-paragraph element (like a table, image, etc.) is encountered,
      // it signifies a structural break, so stop searching for a preceding heading.
      return "";
    }
  }

  // If the loop finishes, no H3 was found before the beginning of the document.
  return "";
}

/**
 * Gets the starting position of a given paragraph in the document.
 * This is used for creating bookmarks.
 *
 * @param {GoogleAppsScript.Document.Paragraph} paragraph The paragraph for which to get the position.
 * @returns {GoogleAppsScript.Document.Position} The position object at the beginning of the paragraph.
 * @throws {Error} If the input is not a Paragraph object.
 */
function getParagraphStartPosition(paragraph) {
  if (!paragraph || paragraph.getType() !== DocumentApp.ElementType.PARAGRAPH) {
    throw new Error("Invalid input: Expected a Paragraph object.");
  }
  const doc = DocumentApp.getActiveDocument();
  // Create a new position at the beginning (offset 0) of the paragraph
  const position = doc.newPosition(paragraph, 0);
  return position;
}


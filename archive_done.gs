
/**
 * Finds all normal lines (including list items) with a '✅' symbol,
 * moves them to the top of the "Done" H1 section, and then refreshes the report.
 * If the "Done" H1 section does not exist, it will be created at the end of the document.
 */
function archiveDoneTasks() {
  const doc = DocumentApp.getActiveDocument();
  const body = doc.getBody();

  Logger.log("--- Starting Archive Done Tasks ---");

  const archivedLines = [];
  let doneSectionParagraphIndex = -1; // To store the index of the "Done" H1 paragraph

  // 1. Find the "Done" H1 section. Iterate from top to bottom.
  // This needs to happen before removing paragraphs, as removal changes indices.
  for (let i = 0; i < body.getNumChildren(); i++) {
    const child = body.getChild(i);
    if (child.getType() === DocumentApp.ElementType.PARAGRAPH) {
      const paragraph = child.asParagraph();
      if (paragraph.getHeading() === DocumentApp.ParagraphHeading.HEADING1 && paragraph.getText().trim() === "Done") {
        doneSectionParagraphIndex = i;
        Logger.log(`Found "Done" H1 section at index: ${doneSectionParagraphIndex}`);
        break; // Found the "Done" H1 section
      }
    }
  }

  // If "Done" H1 section not found, create it at the very end of the document.
  if (doneSectionParagraphIndex === -1) {
    // Appending a new paragraph to the body and setting its heading and text.
    body.appendParagraph("Done").setHeading(DocumentApp.ParagraphHeading.HEADING1);
    // The index of the newly appended paragraph will be `body.getNumChildren() - 1`.
    doneSectionParagraphIndex = body.getNumChildren() - 1;
    Logger.log("Created 'Done' H1 section at the end of the document.");
  }

  // 2. Collect lines with "✅" and remove them, iterating from bottom to top.
  // This iteration order is critical to prevent index issues when removing elements.
  for (let i = body.getNumChildren() - 1; i >= 0; i--) {
    const child = body.getChild(i);

    // Skip the "Done" H1 section itself and the report markers if encountered
    const isParagraph = child.getType() === DocumentApp.ElementType.PARAGRAPH;
    const paragraphText = isParagraph ? child.asParagraph().getText().trim() : '';

    if (paragraphText === "--- GTD Symbol Report ---" ||
      paragraphText === "-------------------------") {
      continue; // Do not process report markers
    }

    // Also skip the "Done" H1 section that we're targeting for insertion
    if (isParagraph && i === doneSectionParagraphIndex && child.asParagraph().getHeading() === DocumentApp.ParagraphHeading.HEADING1 && paragraphText === "Done") {
      continue;
    }

    if (isParagraph) {
      const paragraph = child.asParagraph();
      const heading = paragraph.getHeading();
      const text = paragraph.getText();

      // Check for normal text or list items with "✅"
      if ((heading === DocumentApp.ParagraphHeading.NORMAL ||
        heading === DocumentApp.ParagraphHeading.LIST_ITEM ||
        heading === DocumentApp.ParagraphHeading.UNORDERED_LIST_ITEM) &&
        text.includes("✅")) {

        // Add the paragraph's full text (including new line character) to the beginning
        // of the archivedLines array to maintain original order when inserted later.
        archivedLines.unshift(text + '\n');
        paragraph.removeFromParent(); // Remove from document
        Logger.log(`Archived and removed: "${text.trim()}"`);
      }
    }
  }

  // 3. Insert the collected lines at the top of the "Done" H1 section.
  // We need to re-find the index of the "Done" section after removals,
  // as its index might have changed due to other paragraphs being removed.
  let actualDoneSectionIndex = -1;
  for (let i = 0; i < body.getNumChildren(); i++) {
    const child = body.getChild(i);
    if (child.getType() === DocumentApp.ElementType.PARAGRAPH) {
      const paragraph = child.asParagraph();
      // Use .trim() for robust comparison
      if (paragraph.getHeading() === DocumentApp.ParagraphHeading.HEADING1 && paragraph.getText().trim() === "Done") {
        actualDoneSectionIndex = i;
        Logger.log(`Re-found "Done" H1 section at index: ${actualDoneSectionIndex} for insertion.`);
        break;
      }
    }
  }

  if (actualDoneSectionIndex !== -1) {
    // Insert after the H1 paragraph itself, which is actualDoneSectionIndex + 1
    const insertionPoint = actualDoneSectionIndex + 1;
    for (const line of archivedLines) {
      body.insertParagraph(insertionPoint, line.trim()); // Insert with original formatting
    }
    Logger.log(`Inserted ${archivedLines.length} lines into 'Done' section.`);
  } else {
    // This case should ideally not happen if we created it when not found initially.
    Logger.log("Error: 'Done' H1 section not found for insertion after removal process. This shouldn't happen.");
  }

  // Refresh the report after modification and archiving
  countAndReportSymbols(); // This will update the report at the top

  // Finally, save and close the document ONLY ONCE after all operations
  doc.saveAndClose();
}


package com.gemwallet.android.ui.components

import androidx.compose.ui.text.LinkAnnotation
import androidx.compose.ui.text.font.FontWeight
import org.junit.Assert.assertEquals
import org.junit.Test

class MarkdownTest {
    @Test
    fun bareLinksAreAnnotatedWithoutTrailingPunctuation() {
        val text = parseMarkdownToAnnotatedString("Open https://google.com.")

        assertEquals("Open https://google.com.", text.text)
        assertEquals(listOf("https://google.com"), text.linkUrls())
    }

    @Test
    fun markdownLinksKeepExistingAnnotation() {
        val text = parseMarkdownToAnnotatedString("[Docs](https://docs.gemwallet.com)")

        assertEquals("Docs", text.text)
        assertEquals(listOf("https://docs.gemwallet.com"), text.linkUrls())
    }

    @Test
    fun listLinesStayAsWrittenWithBold() {
        val text = parseMarkdownToAnnotatedString("- **Status:** pending\n- Network")

        assertEquals("- Status: pending\n- Network", text.text)
        assertEquals(listOf("Status:"), text.boldRuns())
    }

    @Test
    fun headingSyntaxIsPlainText() {
        val text = parseMarkdownToAnnotatedString("#12 is your ticket\n# Title")

        assertEquals("#12 is your ticket\n# Title", text.text)
        assertEquals(emptyList<Any>(), text.spanStyles)
    }

    @Test
    fun blockquoteSyntaxIsPlainText() {
        val text = parseMarkdownToAnnotatedString("> Note\nNext")

        assertEquals("> Note\nNext", text.text)
    }

    private fun androidx.compose.ui.text.AnnotatedString.boldRuns(): List<String> = spanStyles.filter { it.item.fontWeight == FontWeight.Bold }.map { text.substring(it.start, it.end) }

    private fun androidx.compose.ui.text.AnnotatedString.linkUrls(): List<String> = getLinkAnnotations(0, length).map { (it.item as LinkAnnotation.Url).url }
}

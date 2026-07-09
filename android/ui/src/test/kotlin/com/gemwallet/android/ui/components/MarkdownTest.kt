package com.gemwallet.android.ui.components

import androidx.compose.ui.text.LinkAnnotation
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

    private fun androidx.compose.ui.text.AnnotatedString.linkUrls(): List<String> {
        return getLinkAnnotations(0, length).map { (it.item as LinkAnnotation.Url).url }
    }
}

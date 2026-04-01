package com.gemwallet.android.ext

import kotlinx.coroutines.flow.take
import kotlinx.coroutines.flow.toList
import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

class TickerFlowTest {

    @Test
    fun `ticker emits after delay`() = runBlocking {
        val results = tickerFlow(50) {}
            .take(2)
            .toList()

        assertEquals(2, results.size)
        assertTrue(results.all { it > 0 })
    }
}

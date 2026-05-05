package com.gemwallet.android.ui.navigation

import androidx.navigation3.runtime.NavEntry
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.ui.models.navigation.RouteArgument
import org.junit.Assert.assertEquals
import org.junit.Test

class RouteArgumentsNavEntryDecoratorTest {

    @Test
    fun contentKey_keepsFirstEntryAndDisambiguatesDuplicateOccurrences() {
        val key = TestRoute("asset")
        val entry = NavEntry<NavKey>(key) {}

        assertEquals(
            key.toString(),
            entry.withOccurrenceContentKey(key, occurrence = 0).contentKey,
        )
        assertEquals(
            "$key#1",
            entry.withOccurrenceContentKey(key, occurrence = 1).contentKey,
        )
    }

    @Test
    fun routeArguments_dropsNullValues() {
        assertEquals(
            mapOf("routeArguments" to mapOf(RouteArgument.AssetId.key to "bitcoin")),
            routeArguments(
                RouteArgument.AssetId to "bitcoin",
                RouteArgument.NftAssetId to null,
            ),
        )
    }

    private data class TestRoute(val id: String) : NavKey
}

package com.gemwallet.android.integration

import androidx.navigation3.runtime.NavKey
import androidx.navigation3.runtime.serialization.NavKeySerializer
import androidx.savedstate.serialization.decodeFromSavedState
import androidx.savedstate.serialization.encodeToSavedState
import androidx.test.ext.junit.runners.AndroidJUnit4
import com.gemwallet.android.ui.navigation.routes.AssetsResultsRoute
import org.junit.Assert.assertEquals
import org.junit.Test
import org.junit.runner.RunWith
import uniffi.gemstone.GemSearchScope

@RunWith(AndroidJUnit4::class)
class AssetsResultsRouteTest {

    @Test
    fun aRestoredBackStackKeepsTheSearchScope() {
        listOf(
            AssetsResultsRoute(query = "eth", scope = GemSearchScope.All),
            AssetsResultsRoute(query = "", scope = GemSearchScope.List("trending"), title = "Trending"),
        ).forEach { route ->
            assertEquals(route, decodeFromSavedState(NavKeySerializer<NavKey>(), encodeToSavedState(NavKeySerializer<NavKey>(), route)))
        }
    }
}

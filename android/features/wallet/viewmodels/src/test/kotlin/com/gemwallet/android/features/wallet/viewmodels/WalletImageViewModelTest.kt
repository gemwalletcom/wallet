package com.gemwallet.android.features.wallet.viewmodels

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.data.services.store.queries.NFTQuery
import com.gemwallet.android.data.services.store.queries.WalletQuery
import com.gemwallet.android.testkit.MainDispatcherRule
import com.gemwallet.android.ui.models.navigation.RouteArgument
import io.mockk.coEvery
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.runTest
import org.junit.After
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import uniffi.gemstone.GemAvatarList
import uniffi.gemstone.GemListPhase
import uniffi.gemstone.GemWalletServiceInterface

@OptIn(ExperimentalCoroutinesApi::class)
class WalletImageViewModelTest {

    private val dispatcher = StandardTestDispatcher()
    private val models = mutableListOf<ViewModel>()

    @get:Rule
    val mainDispatcherRule = MainDispatcherRule(dispatcher)

    @After
    fun tearDown() {
        models.forEach { it.viewModelScope.cancel() }
        models.clear()
    }

    @Test
    fun `an avatar Core refuses shows an error until it is cleared`() = runTest(dispatcher) {
        val service: GemWalletServiceInterface = mockk {
            coEvery { setAvatarImageUrl(any(), any()) } throws IllegalStateException("no image")
            every { avatarList(any()) } returns GemAvatarList(emptyList(), GemListPhase.Rows)
        }
        val walletQuery: WalletQuery = mockk { every { this@mockk(any()) } returns flowOf(null) }
        val nfts: NFTQuery = mockk { every { this@mockk.invoke(any(), any()) } returns flowOf(emptyList()) }
        val route = SavedStateHandle(mapOf(RouteArgument.WalletId.key to "multicoin_0xabc"))
        val model = WalletImageViewModel(walletQuery, nfts, service, route, dispatcher, mockk(relaxed = true)).also { models.add(it) }

        model.setNftImage("https://example.com/a.png").join()

        assertTrue(model.error.value != null)
        model.clearError()
        assertNull(model.error.value)
    }
}

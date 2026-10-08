package com.gemwallet.android.features.transfer.viewmodels.recipient

import androidx.lifecycle.SavedStateHandle
import com.gemwallet.android.application.session.cases.GetCurrentWalletId
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.features.transfer.viewmodels.recipient.models.RecipientUIState
import com.gemwallet.android.testkit.MainDispatcherRule
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.ui.models.navigation.RouteArgument
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.emptyFlow
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Rule
import org.junit.Test
import uniffi.gemstone.GemRecipient
import uniffi.gemstone.GemRecipientServiceInterface
import uniffi.gemstone.GemRecipientType
import uniffi.gemstone.InternalException

@OptIn(ExperimentalCoroutinesApi::class)
class RecipientViewModelTest {

    private val dispatcher = UnconfinedTestDispatcher()
    private val asset = mockAsset()

    @get:Rule
    val mainDispatcherRule = MainDispatcherRule(dispatcher)

    @Test
    fun `an unexpected failure picking a recipient shows it in the address field`() = runTest(dispatcher) {
        val service = mockk<GemRecipientServiceInterface>(relaxed = true)
        every { service.select(any(), any()) } throws InternalException("panic")
        val viewModel = RecipientViewModel(
            walletAddressItemsQuery = mockk(relaxed = true),
            contactsQuery = mockk(relaxed = true),
            getCurrentWalletId = mockk<GetCurrentWalletId> { every { this@mockk() } returns emptyFlow() },
            assetQuery = mockk(relaxed = true),
            savedStateHandle = SavedStateHandle(mapOf(RouteArgument.AssetId.key to asset.id.toIdentifier())),
            service = service,
            nameService = mockk(relaxed = true),
            context = mockk(relaxed = true),
            ioDispatcher = dispatcher,
        )

        viewModel.onDestination(
            recipient = RecipientUIState.Ready(asset, GemRecipientType.Asset(asset.toGem())),
            destination = GemRecipient(address = "address"),
            amountAction = {},
            confirmAction = {},
        )

        assertEquals("panic", viewModel.addressError.value)
    }
}

package com.gemwallet.android.features.contacts.viewmodels

import androidx.lifecycle.SavedStateHandle
import com.gemwallet.android.testkit.MainDispatcherRule
import com.gemwallet.android.testkit.NameServiceMock
import com.gemwallet.android.testkit.mockGemContactAddressSession
import com.gemwallet.android.testkit.mockGemContactScannedAddress
import com.gemwallet.android.testkit.mockGemContactSession
import com.gemwallet.android.testkit.mockGemRecipientValidation
import com.wallet.core.primitives.Chain
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import uniffi.gemstone.GemContactEditorServiceInterface
import uniffi.gemstone.GemNameRecordState
import uniffi.gemstone.GemNameServiceInterface

@OptIn(ExperimentalCoroutinesApi::class)
class ContactEditorViewModelTest {

    private val dispatcher = StandardTestDispatcher()

    @get:Rule
    val mainDispatcherRule = MainDispatcherRule(dispatcher)

    private val service = mockk<GemContactEditorServiceInterface> {
        every { newSession(any(), any()) } returns mockGemContactSession()
        every { newAddressSession(any(), any()) } returns mockGemContactAddressSession(chain = Chain.Bitcoin.string)
        every { scannedAddress(any()) } returns mockGemContactScannedAddress(address = "UQaddress", chain = Chain.Ton.string, memo = "1234567")
    }

    private val tonOnlyNames = object : GemNameServiceInterface by NameServiceMock() {
        override fun validateRecipient(chain: uniffi.gemstone.Chain, input: String, state: GemNameRecordState) = mockGemRecipientValidation(isValid = chain == Chain.Ton.string, address = input)
    }

    private fun model() = ContactEditorViewModel(
        contactQuery = mockk(),
        context = mockk(),
        service = service,
        nameService = tonOnlyNames,
        toastPresenter = mockk(),
        savedStateHandle = SavedStateHandle(),
        ioDispatcher = dispatcher,
    )

    @Test
    fun `a scanned code moves the address to its network and keeps its memo`() = runTest(dispatcher) {
        val model = model()

        model.addAddress()
        model.scanAddress("code")
        advanceUntilIdle()

        val input = checkNotNull(model.uiState.value.addressInput)
        assertEquals(Chain.Ton, input.chain)
        assertEquals("1234567", input.memo)
        assertTrue(input.isAddressValid)
    }
}

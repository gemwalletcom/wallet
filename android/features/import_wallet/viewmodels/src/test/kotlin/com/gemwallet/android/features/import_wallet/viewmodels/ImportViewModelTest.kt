package com.gemwallet.android.features.import_wallet.viewmodels

import com.gemwallet.android.blockchain.operators.gemstone.GemFindPhraseWord
import com.gemwallet.android.blockchain.operators.gemstone.GemValidatePhraseOperator
import com.gemwallet.android.serializer.toJson
import io.mockk.coEvery
import uniffi.gemstone.GemNameServiceInterface
import com.gemwallet.android.ext.networkName
import com.gemwallet.android.model.ImportType
import com.gemwallet.android.ui.models.name.NameRecordState
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.NameProvider
import com.wallet.core.primitives.NameRecord
import com.wallet.core.primitives.WalletType
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkStatic
import io.mockk.unmockkAll
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Test

@OptIn(ExperimentalCoroutinesApi::class)
class ImportViewModelTest {

    private val chain = Chain.Ethereum
    private val record = NameRecord(
        name = "vitalik.eth",
        chain = chain,
        address = "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
        provider = NameProvider.Ens,
    )

    private class NameRequests(private val result: NameRecord?) {
        val requests = mutableListOf<Pair<String, Chain>>()

        fun service(): GemNameServiceInterface = mockk(relaxed = true) {
            every { isNameSupported(any()) } answers { firstArg<String>().split(".").size >= 2 }
            every { nameRecordDebounceMilliseconds() } returns 500u
            coEvery { getNameRecord(any(), any()) } answers {
                requests.add(firstArg<String>() to Chain.entries.first { it.string == secondArg<String>() })
                result?.toJson()
            }
        }
    }

    private fun viewModel(nameService: GemNameServiceInterface) = ImportViewModel(
        service = mockk(relaxed = true),
        nameService = nameService,
        validatePhrase = GemValidatePhraseOperator(),
        findPhraseWord = GemFindPhraseWord(),
    )

    @Before
    fun setUp() {
        mockkStatic("com.gemwallet.android.ext.ChainKt")
        every { any<Chain>().networkName() } returns "Ethereum"
    }

    @After
    fun tearDown() {
        unmockkAll()
        Dispatchers.resetMain()
    }

    @Test
    fun privateKeyInputNeverReachesTheResolver() = runTest {
        Dispatchers.setMain(StandardTestDispatcher(testScheduler))
        val addressInput = NameRequests(record)
        val viewModel = viewModel(addressInput.service())

        viewModel.importSelect(ImportType(WalletType.PrivateKey, chain)).join()
        advanceUntilIdle()
        viewModel.onInput("vitalik.eth")
        advanceUntilIdle()

        assertEquals(emptyList<Pair<String, Chain>>(), addressInput.requests)
        assertEquals(NameRecordState.None, viewModel.nameResolveState.value)
    }

    @Test
    fun viewAddressInputResolves() = runTest {
        Dispatchers.setMain(StandardTestDispatcher(testScheduler))
        val addressInput = NameRequests(record)
        val viewModel = viewModel(addressInput.service())

        viewModel.importSelect(ImportType(WalletType.View, chain)).join()
        advanceUntilIdle()
        viewModel.onInput("vitalik.eth")
        advanceUntilIdle()

        assertEquals(listOf("vitalik.eth" to chain), addressInput.requests)
        assertEquals(NameRecordState.Complete(record), viewModel.nameResolveState.value)
    }
}

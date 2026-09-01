package com.gemwallet.android.features.import_wallet.viewmodels

import com.gemwallet.android.blockchain.operators.gemstone.GemFindPhraseWord
import com.gemwallet.android.blockchain.operators.gemstone.GemValidatePhraseOperator
import com.gemwallet.android.application.recipient.cases.GetNameRecord
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

    private class FakeGetNameRecord(private val result: NameRecord?) : GetNameRecord {
        val requests = mutableListOf<Pair<String, Chain>>()

        override suspend fun getNameRecord(name: String, chain: Chain): NameRecord? {
            requests.add(name to chain)
            return result
        }

        override fun isNameSupported(name: String): Boolean = name.split(".").size >= 2
    }

    private fun viewModel(getNameRecord: GetNameRecord) = ImportViewModel(
        walletService = mockk(relaxed = true),
        importWalletService = mockk(relaxed = true),
        setCurrentWallet = mockk(relaxed = true),
        validatePhrase = GemValidatePhraseOperator(),
        findPhraseWord = GemFindPhraseWord(),
        getNameRecord = getNameRecord,
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
        val getNameRecord = FakeGetNameRecord(record)
        val viewModel = viewModel(getNameRecord)

        viewModel.importSelect(ImportType(WalletType.PrivateKey, chain)).join()
        advanceUntilIdle()
        viewModel.onInput("vitalik.eth")
        advanceUntilIdle()

        assertEquals(emptyList<Pair<String, Chain>>(), getNameRecord.requests)
        assertEquals(NameRecordState.None, viewModel.nameResolveState.value)
    }

    @Test
    fun viewAddressInputResolves() = runTest {
        Dispatchers.setMain(StandardTestDispatcher(testScheduler))
        val getNameRecord = FakeGetNameRecord(record)
        val viewModel = viewModel(getNameRecord)

        viewModel.importSelect(ImportType(WalletType.View, chain)).join()
        advanceUntilIdle()
        viewModel.onInput("vitalik.eth")
        advanceUntilIdle()

        assertEquals(listOf("vitalik.eth" to chain), getNameRecord.requests)
        assertEquals(NameRecordState.Complete(record), viewModel.nameResolveState.value)
    }
}

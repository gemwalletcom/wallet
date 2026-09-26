package com.gemwallet.android.features.transfer.viewmodels.confirm

import android.content.Context
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.features.transfer.viewmodels.confirm.models.ConfirmHeaderUIModel
import com.gemwallet.android.features.transfer.viewmodels.confirm.models.confirmHeader
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockGemAssetIcon
import com.gemwallet.android.testkit.mockGemFormattedNumber
import com.gemwallet.android.testkit.mockGemHeaderAmount
import io.mockk.mockk
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Test
import uniffi.gemstone.GemApprovalValue
import uniffi.gemstone.GemConfirmHeader
import uniffi.gemstone.GemNumberUnit
import uniffi.gemstone.GemSimulationValue
import uniffi.gemstone.GemTransactionHeader
import java.util.Locale

class ConfirmHeaderUIModelTest {
    private var originalLocale: Locale = Locale.getDefault()
    private val context: Context = mockk(relaxed = true)

    @Before
    fun setUp() {
        originalLocale = Locale.getDefault()
        Locale.setDefault(Locale.US)
    }

    @After
    fun tearDown() {
        Locale.setDefault(originalLocale)
    }

    @Test
    fun `an approval header reads the value Core resolved`() {
        val asset = mockAsset()
        val header = GemConfirmHeader.Value(GemSimulationValue(asset.toGem(), GemApprovalValue.Unlimited, mockGemAssetIcon()))

        val model = confirmHeader(header, context = context)

        assertEquals(ConfirmHeaderUIModel.Simulation::class, model::class)
    }

    @Test
    fun `a placeholder keeps the head in place until the value arrives`() {
        val asset = mockAsset()
        val header = GemConfirmHeader.Placeholder(mockGemAssetIcon())

        val model = confirmHeader(header, context = context)

        assertEquals(ConfirmHeaderUIModel.Placeholder::class, model::class)
    }

    @Test
    fun `a payment request reserves the head's height without showing it`() {
        val asset = mockAsset()
        val amount = mockGemHeaderAmount(asset = asset.toGem(), amount = mockGemFormattedNumber(value = 1.0, unit = GemNumberUnit.Symbol(asset.symbol)), fiat = mockGemFormattedNumber(value = 2.0))
        val header = GemConfirmHeader.Reserved(GemTransactionHeader.Amount(amount))

        assertEquals(ConfirmHeaderUIModel.ReservedSpace(amount.icon), confirmHeader(header, context = context))
    }

    @Test
    fun `a transaction header draws the amount Core carried`() {
        val asset = mockAsset()
        val header = GemConfirmHeader.Transaction(
            GemTransactionHeader.Amount(mockGemHeaderAmount(asset = asset.toGem(), amount = mockGemFormattedNumber(value = 1.0, unit = GemNumberUnit.Symbol(asset.symbol)), fiat = mockGemFormattedNumber(value = 2.0))),
        )

        val model = confirmHeader(header, context = context)

        assertEquals(asset, (model as ConfirmHeaderUIModel.Amount).asset)
    }
}

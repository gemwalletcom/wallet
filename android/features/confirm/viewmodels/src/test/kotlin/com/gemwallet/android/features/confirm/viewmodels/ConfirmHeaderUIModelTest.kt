package com.gemwallet.android.features.confirm.viewmodels

import android.content.Context
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.features.confirm.viewmodels.models.ConfirmHeaderUIModel
import com.gemwallet.android.features.confirm.viewmodels.models.confirmHeader
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockFormattedNumber
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
        val header = GemConfirmHeader.Value(GemSimulationValue(asset.toGem(), GemApprovalValue.Unlimited))

        val model = confirmHeader(header, context = context)

        assertEquals(ConfirmHeaderUIModel.Simulation::class, model::class)
    }

    @Test
    fun `a placeholder keeps the head in place until the value arrives`() {
        val asset = mockAsset()
        val header = GemConfirmHeader.Placeholder(asset.id.toIdentifier())

        val model = confirmHeader(header, context = context)

        assertEquals(ConfirmHeaderUIModel.Placeholder::class, model::class)
    }

    @Test
    fun `a payment request reserves the head's height without showing it`() {
        val asset = mockAsset()
        val header = GemConfirmHeader.Reserved(GemTransactionHeader.Amount(mockGemHeaderAmount(asset = asset.toGem(), amount = mockFormattedNumber(1.0, GemNumberUnit.Symbol(asset.symbol)), fiat = mockFormattedNumber(2.0))))

        assertEquals(ConfirmHeaderUIModel.ReservedSpace(asset), confirmHeader(header, context = context))
    }

    @Test
    fun `a transaction header draws the amount Core carried`() {
        val asset = mockAsset()
        val header = GemConfirmHeader.Transaction(GemTransactionHeader.Amount(mockGemHeaderAmount(asset = asset.toGem(), amount = mockFormattedNumber(1.0, GemNumberUnit.Symbol(asset.symbol)), fiat = mockFormattedNumber(2.0))))

        val model = confirmHeader(header, context = context)

        assertEquals(asset, (model as ConfirmHeaderUIModel.Amount).asset)
    }
}

package com.gemwallet.android.data.coordinators.perpetuals

import com.gemwallet.android.data.services.gemstone.stores.GemstonePerpetualStore
import com.gemwallet.android.domains.asset.aggregates.trailingValue
import com.gemwallet.android.model.text
import com.gemwallet.android.testkit.mockPerpetual
import com.gemwallet.android.testkit.mockPerpetualData
import com.wallet.core.primitives.Currency
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.runTest
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Before
import org.junit.Test
import uniffi.gemstone.GemCurrencyStyle
import uniffi.gemstone.GemLocalizedText
import uniffi.gemstone.GemRowText
import uniffi.gemstone.GemValueTone
import uniffi.gemstone.formattedCurrency
import java.util.Locale

class GetPerpetualsImplTest {
    private val defaultLocale = Locale.getDefault()

    @Before
    fun setup() {
        Locale.setDefault(Locale.US)
    }

    @After
    fun tearDown() {
        Locale.setDefault(defaultLocale)
    }

    @Test
    fun rowsCarryFormattedPriceAndVolume() = runTest {
        val store = mockk<GemstonePerpetualStore> {
            every { observePerpetuals(any()) } returns flowOf(
                listOf(
                    mockPerpetualData(perpetual = mockPerpetual(price = 95420.5, pricePercentChange24h = 2.5, volume24h = 15_000.0)),
                    mockPerpetualData(perpetual = mockPerpetual(price = 0.0, pricePercentChange24h = -1.25, volume24h = 0.0)),
                ),
            )
        }

        val rows = GetPerpetualsImpl(store).getPerpetuals(null).first()

        val quoted = rows[0].row
        assertEquals("$95,420.50", quoted.subtitle.number)
        assertEquals("+2.50%", quoted.subtitleExtra.number)
        assertEquals(GemValueTone.POSITIVE, quoted.subtitleExtra?.tone)
        assertEquals(formattedCurrency(15_000.0, Currency.USD.string, GemCurrencyStyle.ABBREVIATED).text(), quoted.trailingValue.number)
        assertNull("a market nobody quoted shows no price", rows[1].row.subtitle)
        assertNull("its change goes with it", rows[1].row.subtitleExtra)
    }
}

private val GemRowText?.number: String?
    get() = (this?.text as? GemLocalizedText.Number)?.number?.text()

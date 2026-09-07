package com.gemwallet.android.data.coordinators.perpetuals

import com.gemwallet.android.data.services.gemstone.stores.GemstonePerpetualStore
import com.gemwallet.android.domains.price.ValueDirection
import com.gemwallet.android.model.CurrencyFormatter
import com.wallet.core.primitives.Currency
import com.gemwallet.android.testkit.mockPerpetual
import com.gemwallet.android.testkit.mockPerpetualData
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.runTest
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Test
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

        assertEquals("$95,420.50", rows[0].price.valueFormatted)
        assertEquals("+2.50%", rows[0].price.changePercentageFormatted)
        assertEquals(ValueDirection.Up, rows[0].price.state)
        assertEquals(CurrencyFormatter(type = CurrencyFormatter.Type.Abbreviated, currency = Currency.USD).string(15_000.0), rows[0].volume)
        assertEquals("$0.00", rows[1].price.valueFormatted)
        assertEquals("-1.25%", rows[1].price.changePercentageFormatted)
        assertEquals(ValueDirection.Down, rows[1].price.state)
    }
}

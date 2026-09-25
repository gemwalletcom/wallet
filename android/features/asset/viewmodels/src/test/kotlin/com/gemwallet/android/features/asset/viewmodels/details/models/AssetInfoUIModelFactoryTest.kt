package com.gemwallet.android.features.asset.viewmodels.details.models

import android.content.Context
import com.gemwallet.android.ext.asset
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.text
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetInfo
import com.gemwallet.android.testkit.mockChainAssetInfo
import com.gemwallet.android.testkit.mockFormattedNumber
import com.gemwallet.android.testkit.mockGemAssetDetails
import com.gemwallet.android.testkit.mockGemAssetDetailsState
import com.gemwallet.android.ui.R
import com.wallet.core.primitives.Chain
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkStatic
import io.mockk.unmockkStatic
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Test
import uniffi.gemstone.GemAssetBalanceRow
import uniffi.gemstone.GemAssetDetailRow
import uniffi.gemstone.GemAssetDetailSection
import uniffi.gemstone.GemBalanceRow
import uniffi.gemstone.GemBalanceRowValue
import uniffi.gemstone.GemFormattedNumber
import uniffi.gemstone.GemListRow
import uniffi.gemstone.GemListRowIcon
import uniffi.gemstone.GemListRowTitle
import uniffi.gemstone.GemListSectionTitle
import uniffi.gemstone.GemNumberUnit
import uniffi.gemstone.GemValueTone
import java.math.BigInteger

class AssetInfoUIModelFactoryTest {

    @Before
    fun setUp() {
        mockkStatic("com.gemwallet.android.ext.ChainKt")
        every { Chain.Cosmos.asset() } returns mockAsset(chain = Chain.Cosmos, name = "Cosmos")
        every { Chain.Solana.asset() } returns mockAsset(chain = Chain.Solana, name = "Solana")
        every { Chain.Tron.asset() } returns mockAsset(chain = Chain.Tron, name = "Tron")
        every { Chain.Bitcoin.asset() } returns mockAsset(chain = Chain.Bitcoin, name = "Bitcoin")
    }

    @After
    fun tearDown() = unmockkStatic("com.gemwallet.android.ext.ChainKt")

    @Test
    fun `balance rows show the values core formatted`() {
        val staked = mockFormattedNumber(value = 2.0, unit = GemNumberUnit.Symbol(symbol = "ATOM"))
        val apr = mockFormattedNumber(value = 5.0, unit = GemNumberUnit.Percent)
        val asset = mockAsset(chain = Chain.Cosmos)
        val section = model(
            mockAssetInfo(asset = asset, owner = null),
            sections = listOf(
                GemAssetDetailSection(
                    GemListSectionTitle.BALANCES,
                    listOf(
                        GemAssetBalanceRow(GemBalanceRow.Staked(BigInteger("2000000")), GemBalanceRowValue.Amount(staked)),
                        GemAssetBalanceRow(GemBalanceRow.Reserved(BigInteger("500000"), "https://reserve"), GemBalanceRowValue.Amount(staked)),
                        GemAssetBalanceRow(GemBalanceRow.Staked(BigInteger.ZERO), GemBalanceRowValue.Apr(apr)),
                        GemAssetBalanceRow(GemBalanceRow.Staked(BigInteger.ZERO), GemBalanceRowValue.Apr(null)),
                    ).map { GemAssetDetailRow.Balance(it) },
                ),
            ),
        ).sections.single()
        val balances = section.rows.filterIsInstance<AssetInfoUIModel.RowUIModel.Balance>()

        assertEquals(R.string.asset_balances, section.title)
        assertEquals(4, balances.size)
        val stake = AssetDetailsAction.Stake(asset.id)
        assertEquals(listOf(stake, AssetDetailsAction.OpenUrl("https://reserve"), stake, stake), balances.map { it.action })
        assertEquals(
            listOf(staked.text(), staked.text(), "${R.string.stake_apr}${apr.text()}", "${R.string.stake_apr}"),
            balances.map { it.model.subtitle },
        )
    }

    @Test
    fun `the price row shows what core quoted and nothing when nobody quoted`() {
        val price = mockFormattedNumber(value = 1234.5, unit = GemNumberUnit.Currency(code = "USD"))
        val change = mockFormattedNumber(value = -2.5, unit = GemNumberUnit.Percent, tone = GemValueTone.NEGATIVE)
        val asset = mockAsset()
        val assetInfo = mockAssetInfo(asset = asset, owner = null)
        val quoted = listOf(GemAssetDetailSection(GemListSectionTitle.NONE, listOf(GemAssetDetailRow.Row(GemListRow.Quote(GemListRowTitle.PRICE, price, change)))))
        val unquoted = listOf(GemAssetDetailSection(GemListSectionTitle.NONE, listOf(GemAssetDetailRow.Row(GemListRow.Quote(GemListRowTitle.PRICE, null, null)))))

        val quotedRow = model(assetInfo, sections = quoted).sections.first().rows.first() as AssetInfoUIModel.RowUIModel.Row
        val unquotedRow = model(assetInfo, sections = unquoted).sections.first().rows.first() as AssetInfoUIModel.RowUIModel.Row

        assertEquals(GemListRow.Quote(GemListRowTitle.PRICE, price, change), quotedRow.row)
        assertEquals(AssetDetailsAction.OpenChart(asset.id), quotedRow.action)
        assertEquals(GemListRow.Quote(GemListRowTitle.PRICE, null, null), unquotedRow.row)
    }

    @Test
    fun `sections keep the order and titles core decided`() {
        val link = GemListRow.Link(GemListRowTitle.PIN, null, GemListRowIcon.PIN)
        val asset = mockAsset()
        val sections = model(
            mockAssetInfo(asset = asset, owner = null),
            sections = listOf(
                GemAssetDetailSection(GemListSectionTitle.MANAGE, listOf(GemAssetDetailRow.Row(link))),
                GemAssetDetailSection(
                    GemListSectionTitle.NONE,
                    listOf(
                        GemAssetDetailRow.Row(GemListRow.Quote(GemListRowTitle.PRICE, null, null)),
                        GemAssetDetailRow.Row(GemListRow.Network(GemListRowTitle.NETWORK, "ethereum", "Ethereum (ERC20)")),
                    ),
                ),
            ),
        ).sections

        assertEquals(listOf(R.string.common_manage, null), sections.map { it.title })
        assertEquals("the pin row carries the action the view only forwards", AssetDetailsAction.Pin, (sections.first().rows.first() as AssetInfoUIModel.RowUIModel.Row).action)
        assertEquals(
            listOf(
                listOf(AssetInfoUIModel.RowUIModel.Row(link, AssetDetailsAction.Pin)),
                listOf(
                    AssetInfoUIModel.RowUIModel.Row(GemListRow.Quote(GemListRowTitle.PRICE, null, null), AssetDetailsAction.OpenChart(asset.id)),
                    AssetInfoUIModel.RowUIModel.Row(GemListRow.Network(GemListRowTitle.NETWORK, "ethereum", "Ethereum (ERC20)"), null),
                ),
            ),
            sections.map { it.rows },
        )
    }

    private val context = mockk<Context> {
        every { getString(any()) } answers { firstArg<Int>().toString() }
        every { getString(any(), *anyVararg()) } answers { "${firstArg<Int>()}${(args[1] as Array<*>).joinToString("")}" }
    }

    private fun model(
        assetInfo: AssetInfo,
        sections: List<GemAssetDetailSection> = emptyList(),
        fiatValue: GemFormattedNumber? = null,
        balanceValue: GemFormattedNumber = mockFormattedNumber(value = 0.0, unit = GemNumberUnit.Symbol(symbol = assetInfo.asset.symbol)),
    ) = AssetInfoUIModelFactory(context).create(
        mockChainAssetInfo(assetInfo),
        mockGemAssetDetails(assetInfo.asset, mockGemAssetDetailsState(showsBanners = true), sections, fiatValue, balanceValue),
    )
}

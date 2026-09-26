package com.gemwallet.android.features.assets.viewmodels.asset.models

import android.content.Context
import com.gemwallet.android.ext.asset
import com.gemwallet.android.model.text
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetData
import com.gemwallet.android.testkit.mockAssetId
import com.gemwallet.android.testkit.mockChainAssetData
import com.gemwallet.android.testkit.mockGemAssetDetails
import com.gemwallet.android.testkit.mockGemAssetDetailsState
import com.gemwallet.android.testkit.mockGemFormattedNumber
import com.gemwallet.android.ui.R
import com.wallet.core.primitives.AssetData
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
import uniffi.gemstone.GemListRow
import uniffi.gemstone.GemListRowIcon
import uniffi.gemstone.GemListRowTitle
import uniffi.gemstone.GemListSectionTitle
import uniffi.gemstone.GemNumberUnit
import uniffi.gemstone.GemRowTap
import uniffi.gemstone.GemValueTone
import java.math.BigInteger

class AssetUIStateFactoryTest {

    @Before
    fun setUp() {
        mockkStatic("com.gemwallet.android.ext.ChainKt")
        every { Chain.Cosmos.asset() } returns mockAsset(id = mockAssetId(chain = Chain.Cosmos), name = "Cosmos")
        every { Chain.Solana.asset() } returns mockAsset(id = mockAssetId(chain = Chain.Solana), name = "Solana")
        every { Chain.Tron.asset() } returns mockAsset(id = mockAssetId(chain = Chain.Tron), name = "Tron")
        every { Chain.Bitcoin.asset() } returns mockAsset(id = mockAssetId(chain = Chain.Bitcoin), name = "Bitcoin")
    }

    @After
    fun tearDown() = unmockkStatic("com.gemwallet.android.ext.ChainKt")

    @Test
    fun `balance rows show the values core formatted`() {
        val staked = mockGemFormattedNumber(value = 2.0, unit = GemNumberUnit.Symbol(symbol = "ATOM"))
        val apr = mockGemFormattedNumber(value = 5.0, unit = GemNumberUnit.Percent)
        val asset = mockAsset(id = mockAssetId(chain = Chain.Cosmos))
        val section = model(
            mockAssetData(asset = asset),
            sections = listOf(
                GemAssetDetailSection(
                    GemListSectionTitle.BALANCES,
                    listOf(
                        GemAssetDetailRow.Balance(GemAssetBalanceRow(GemBalanceRow.Staked(BigInteger("2000000")), GemBalanceRowValue.Amount(staked)), GemRowTap.Stake),
                        GemAssetDetailRow.Balance(GemAssetBalanceRow(GemBalanceRow.Reserved(BigInteger("500000"), "https://reserve"), GemBalanceRowValue.Amount(staked)), GemRowTap.Explorer("https://reserve")),
                        GemAssetDetailRow.Balance(GemAssetBalanceRow(GemBalanceRow.Staked(BigInteger.ZERO), GemBalanceRowValue.Apr(apr)), GemRowTap.Stake),
                        GemAssetDetailRow.Balance(GemAssetBalanceRow(GemBalanceRow.Staked(BigInteger.ZERO), GemBalanceRowValue.Apr(null)), GemRowTap.Stake),
                    ),
                ),
            ),
        ).sections.single()
        val balances = section.rows.filterIsInstance<AssetDetailRowUIModel.Balance>()

        assertEquals(R.string.asset_balances, section.title)
        assertEquals(4, balances.size)
        val stake = AssetAction.Stake(asset.id)
        assertEquals(listOf(stake, AssetAction.OpenUrl("https://reserve"), stake, stake), balances.map { it.action })
        assertEquals(
            listOf(staked.text(), staked.text(), "${R.string.stake_apr}${apr.text()}", "${R.string.stake_apr}"),
            balances.map { it.model.subtitle },
        )
    }

    @Test
    fun `the price row shows what core quoted and nothing when nobody quoted`() {
        val price = mockGemFormattedNumber(value = 1234.5, unit = GemNumberUnit.Currency(code = "USD"))
        val change = mockGemFormattedNumber(value = -2.5, unit = GemNumberUnit.Percent, tone = GemValueTone.NEGATIVE)
        val asset = mockAsset()
        val assetInfo = mockAssetData(asset = asset)
        val quoted = listOf(GemAssetDetailSection(GemListSectionTitle.NONE, listOf(GemAssetDetailRow.Row(GemListRow.Quote(GemListRowTitle.PRICE, price, change), GemRowTap.Price))))
        val unquoted = listOf(GemAssetDetailSection(GemListSectionTitle.NONE, listOf(GemAssetDetailRow.Row(GemListRow.Quote(GemListRowTitle.PRICE, null, null), GemRowTap.Price))))

        val quotedRow = model(assetInfo, sections = quoted).sections.first().rows.first() as AssetDetailRowUIModel.Row
        val unquotedRow = model(assetInfo, sections = unquoted).sections.first().rows.first() as AssetDetailRowUIModel.Row

        assertEquals(GemListRow.Quote(GemListRowTitle.PRICE, price, change), quotedRow.row)
        assertEquals(AssetAction.OpenChart(asset.id), quotedRow.action)
        assertEquals(GemListRow.Quote(GemListRowTitle.PRICE, null, null), unquotedRow.row)
    }

    @Test
    fun `sections keep the order and titles core decided`() {
        val link = GemListRow.Link(GemListRowTitle.PIN, null, GemListRowIcon.PIN, GemRowTap.Pin)
        val asset = mockAsset()
        val sections = model(
            mockAssetData(asset = asset),
            sections = listOf(
                GemAssetDetailSection(GemListSectionTitle.MANAGE, listOf(GemAssetDetailRow.Row(link, GemRowTap.Pin))),
                GemAssetDetailSection(
                    GemListSectionTitle.NONE,
                    listOf(
                        GemAssetDetailRow.Row(GemListRow.Quote(GemListRowTitle.PRICE, null, null), GemRowTap.Price),
                        GemAssetDetailRow.Row(GemListRow.Network(GemListRowTitle.NETWORK, "ethereum", "Ethereum (ERC20)"), GemRowTap.Network),
                    ),
                ),
            ),
        ).sections

        assertEquals(listOf(R.string.common_manage, null), sections.map { it.title })
        assertEquals("the pin row carries the action the view only forwards", AssetAction.Pin, (sections.first().rows.first() as AssetDetailRowUIModel.Row).action)
        assertEquals(
            listOf(
                listOf(AssetDetailRowUIModel.Row(link, AssetAction.Pin)),
                listOf(
                    AssetDetailRowUIModel.Row(GemListRow.Quote(GemListRowTitle.PRICE, null, null), AssetAction.OpenChart(asset.id)),
                    AssetDetailRowUIModel.Row(GemListRow.Network(GemListRowTitle.NETWORK, "ethereum", "Ethereum (ERC20)"), null),
                ),
            ),
            sections.map { it.rows },
        )
    }

    private val context = mockk<Context> {
        every { getString(any()) } answers { firstArg<Int>().toString() }
        every { getString(any(), *anyVararg()) } answers { "${firstArg<Int>()}${(args[1] as Array<*>).joinToString("")}" }
    }

    private fun model(assetInfo: AssetData, sections: List<GemAssetDetailSection> = emptyList()) = AssetUIStateFactory(context).create(
        mockChainAssetData(assetData = assetInfo, feeAssetData = assetInfo),
        mockGemAssetDetails(state = mockGemAssetDetailsState(showsBanners = true), sections = sections),
    )
}

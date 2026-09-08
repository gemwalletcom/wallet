package com.gemwallet.android.ui.components.image

import androidx.annotation.DrawableRes
import com.gemwallet.android.domains.asset.providerName
import com.gemwallet.android.ui.R
import com.wallet.core.primitives.FiatProvider
import com.wallet.core.primitives.FiatProviderName
import uniffi.gemstone.SwapProvider

@DrawableRes
fun SwapProvider.iconResource(): Int = when (this) {
    SwapProvider.UNISWAP_V3,
    SwapProvider.UNISWAP_V4 -> R.drawable.swap_provider_uniswap
    SwapProvider.PANCAKESWAP_V3 -> R.drawable.swap_provider_pancakeswap
    SwapProvider.THORCHAIN -> R.drawable.chain_thorchain
    SwapProvider.MAYACHAIN -> R.drawable.chain_mayachain
    SwapProvider.JUPITER -> R.drawable.swap_provider_jupiter
    SwapProvider.ACROSS -> R.drawable.swap_provider_across
    SwapProvider.OKU -> R.drawable.swap_provider_oku
    SwapProvider.WAGMI -> R.drawable.swap_provider_wagmi
    SwapProvider.CETUS_AGGREGATOR,
    SwapProvider.CETUS_CLMM -> R.drawable.swap_provider_cetus
    SwapProvider.STONFI_V2 -> R.drawable.swap_provider_stonfi
    SwapProvider.MAYAN -> R.drawable.swap_provider_mayan
    SwapProvider.CHAINFLIP -> R.drawable.swap_provider_chainflip
    SwapProvider.RELAY -> R.drawable.swap_provider_relay
    SwapProvider.AERODROME -> R.drawable.swap_provider_aerodrome
    SwapProvider.HYPERLIQUID -> R.drawable.chain_hyperliquid
    SwapProvider.NEAR_INTENTS -> R.drawable.swap_provider_near_intents
    SwapProvider.ORCA -> R.drawable.swap_provider_orca
    SwapProvider.PANORA -> R.drawable.swap_provider_panora
    SwapProvider.OKX -> R.drawable.swap_provider_okx
    SwapProvider.SQUID -> R.drawable.swap_provider_squid
    SwapProvider.SWAPS_XYZ -> R.drawable.swap_provider_swaps_xyz
}

@DrawableRes
fun FiatProviderName.iconResource(): Int = when (this) {
    FiatProviderName.Mercuryo -> R.drawable.fiat_provider_mercuryo
    FiatProviderName.Transak -> R.drawable.fiat_provider_transak
    FiatProviderName.MoonPay -> R.drawable.fiat_provider_moonpay
    FiatProviderName.Banxa -> R.drawable.fiat_provider_banxa
    FiatProviderName.Paybis -> R.drawable.fiat_provider_paybis
    FiatProviderName.Flashnet -> R.drawable.fiat_provider_cashapp
}

fun SwapProvider.iconModel(): Any = iconResource()

fun FiatProvider.iconModel(): Any? = providerName()?.iconResource()

fun FiatProviderName.iconModel(): Any = iconResource()

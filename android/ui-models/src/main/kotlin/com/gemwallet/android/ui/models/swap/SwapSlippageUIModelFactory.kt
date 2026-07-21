package com.gemwallet.android.ui.models.swap

import com.wallet.core.primitives.Chain
import uniffi.gemstone.Config
import uniffi.gemstone.getDefaultSlippageBps

object SwapSlippageUIModelFactory {

    fun create(chain: Chain): SwapSlippageUIModel {
        val slippage = Config().getSwapConfig().slippage
        return SwapSlippageUIModel(
            defaultBps = getDefaultSlippageBps(chain.string),
            suggestionsBps = slippage.suggestionsBps,
            minBps = slippage.minBps,
            maxBps = slippage.maxBps,
            highWarningBps = slippage.highWarningBps,
        )
    }
}

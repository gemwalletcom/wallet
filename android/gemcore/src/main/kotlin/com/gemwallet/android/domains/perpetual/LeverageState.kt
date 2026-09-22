package com.gemwallet.android.domains.perpetual

import com.wallet.core.primitives.PerpetualDirection
import uniffi.gemstone.GemPickerOption

data class LeverageState(val current: GemPickerOption, val options: List<GemPickerOption>, val direction: PerpetualDirection)

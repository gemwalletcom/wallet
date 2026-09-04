package com.gemwallet.android.domains.perpetual

import com.gemwallet.android.domains.asset.toGem
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.CancelOrderData
import com.wallet.core.primitives.PerpetualAccountMode
import com.wallet.core.primitives.PerpetualConfirmData
import com.wallet.core.primitives.PerpetualDirection
import com.wallet.core.primitives.PerpetualMarginType
import com.wallet.core.primitives.PerpetualModifyConfirmData
import com.wallet.core.primitives.PerpetualModifyPositionType
import com.wallet.core.primitives.PerpetualOrderType
import com.wallet.core.primitives.PerpetualPosition
import com.wallet.core.primitives.PerpetualReduceData
import com.wallet.core.primitives.PerpetualTriggerOrder
import com.wallet.core.primitives.PerpetualType
import com.wallet.core.primitives.TPSLOrderData
import uniffi.gemstone.PerpetualAccountMode as GemPerpetualAccountMode
import uniffi.gemstone.PerpetualMarginType as GemPerpetualMarginType
import uniffi.gemstone.PerpetualOrderType as GemPerpetualOrderType
import uniffi.gemstone.PerpetualPosition as GemPerpetualPosition
import uniffi.gemstone.PerpetualTriggerOrder as GemPerpetualTriggerOrder
import uniffi.gemstone.CancelOrderData as GemCancelOrderData
import uniffi.gemstone.PerpetualConfirmData as GemPerpetualConfirmData
import uniffi.gemstone.PerpetualDirection as GemPerpetualDirection
import uniffi.gemstone.PerpetualModifyConfirmData as GemPerpetualModifyConfirmData
import uniffi.gemstone.PerpetualModifyPositionType as GemPerpetualModifyPositionType
import uniffi.gemstone.PerpetualReduceData as GemPerpetualReduceData
import uniffi.gemstone.PerpetualType as GemPerpetualType
import uniffi.gemstone.TpslOrderData as GemTpslOrderData

fun PerpetualConfirmData.toGem(): GemPerpetualConfirmData = toJson()

fun PerpetualReduceData.toGem(): GemPerpetualReduceData = toJson()

fun PerpetualModifyConfirmData.toGem(): GemPerpetualModifyConfirmData = toJson()

fun PerpetualModifyPositionType.toGem(): GemPerpetualModifyPositionType = toJson()

fun TPSLOrderData.toGem(): GemTpslOrderData = toJson()

fun CancelOrderData.toGem(): GemCancelOrderData = toJson()

fun PerpetualPosition.toGem(): GemPerpetualPosition = toJson()

fun PerpetualAccountMode.toGem(): GemPerpetualAccountMode = toJson()

fun PerpetualTriggerOrder.toGem(): GemPerpetualTriggerOrder = toJson()



fun PerpetualType.toGem(): GemPerpetualType = toJson()

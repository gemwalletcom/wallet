// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemAssetItemRow
import struct Gemstone.GemAutocloseSession
import GemstonePrimitives
import Primitives

public typealias AutocloseCompletion = (_ takeProfit: String, _ stopLoss: String) -> Void

public enum AutocloseType {
    case modify(PerpetualPositionData, onTransferAction: TransferDataAction)
    case open(GemAutocloseSession, row: GemAssetItemRow, onComplete: AutocloseCompletion)
}

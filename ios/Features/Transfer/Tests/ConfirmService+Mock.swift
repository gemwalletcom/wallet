// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemNameServiceProtocol
import class Gemstone.GemAmountService
import class Gemstone.GemTransferService
import protocol Gemstone.GemTransactionStateServiceProtocol
import struct Gemstone.GemConfirmData
import GemstonePrimitivesTestKit
import GemstoneServices
import GemstoneServicesTestKit
import Primitives
import PrimitivesComponents
import Store
import StoreTestKit
@testable import Transfer
import TransferTestKit
import class Gemstone.GemFeeService
import class Gemstone.GemSimulationFormatter

extension ConfirmService {
    static func mock(
        gemConfirmService: GemConfirmServiceMock = GemConfirmServiceMock(),
        nameService: any GemNameServiceProtocol = GemNameServiceMock(),
        transactionStateService: any GemTransactionStateServiceProtocol = GemTransactionStateServiceMock(),
    ) -> ConfirmService {
        ConfirmService(
            gemConfirmService: gemConfirmService,
            signer: GemTransactionSignerMock(),
            preferencesService: GemPreferencesServiceMock(),
            transactionStateService: transactionStateService,
            recentAssetsService: RecentAssetsService(store: .mock()),
            toastPresenter: ToastPresenter(),
            keystore: KeystoreMock(),
            explorerService: GemExplorerServiceMock(),
            nameService: nameService,
            assetsService: GemAssetsServiceMock(),
            feeService: GemFeeService(),
            transferService: GemTransferService(),
            perpetualService: GemPerpetualServiceMock(),
        )
    }
}

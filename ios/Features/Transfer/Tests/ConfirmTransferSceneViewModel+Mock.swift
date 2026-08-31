// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemNameServiceProtocol
import class Gemstone.GemAmountService
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
import class Gemstone.GemSimulationFormatter

@MainActor
extension ConfirmTransferSceneViewModel {
    static func mock(
        request: ConfirmTransferRequest? = nil,
        wallet: Wallet? = nil,
        data: TransferData = .mock(),
        simulation: SimulationResult? = nil,
        gemConfirmService: GemConfirmServiceMock = GemConfirmServiceMock(),
        nameService: any GemNameServiceProtocol = GemNameServiceMock(),
        transactionStateService: any GemTransactionStateServiceProtocol = GemTransactionStateServiceMock(),
        onComplete: VoidAction = nil,
    ) -> ConfirmTransferSceneViewModel {
        ConfirmTransferSceneViewModel(
            request: request ?? ConfirmTransferRequest(
                wallet: wallet ?? .mock(accounts: [.mock(chain: data.chain)]),
                data: data,
                simulation: simulation,
            ),
            service: GemConfirmTransferServiceMock(
                confirm: gemConfirmService,
                names: nameService,
                transactionState: transactionStateService,
            ),
            signer: GemTransactionSignerMock(),
            keystore: KeystoreMock(),
            recentAssetsService: RecentAssetsService(store: .mock()),
            toastPresenter: ToastPresenter(),
            preferencesService: GemPreferencesServiceMock(),
            onComplete: onComplete,
        )
    }
}

// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemConfirmData
import struct Gemstone.GemConfirmLoad
import protocol Gemstone.GemTransactionStateServiceProtocol
import struct Gemstone.GemTransferData
import GemstonePrimitives
import GemstonePrimitivesTestKit
import GemstoneServices
import GemstoneServicesTestKit
import Primitives
import PrimitivesComponents
import Store
import StoreTestKit
@testable import Transfer
import TransferTestKit

@MainActor
extension ConfirmTransferSceneViewModel {
    static func mock(
        request: ConfirmTransferRequest? = nil,
        wallet: Wallet? = nil,
        data: GemTransferData = .mock(),
        simulation: SimulationResult? = nil,
        gemConfirmService: GemConfirmServiceMock = GemConfirmServiceMock(),
        load: Result<GemConfirmLoad, any Error> = .success(.mock()),
        transactionStateService: any GemTransactionStateServiceProtocol = GemTransactionStateServiceMock(),
        onComplete: VoidAction = nil,
    ) -> ConfirmTransferSceneViewModel {
        let wallet = wallet ?? .mock(accounts: [.mock(chain: data.chain)])
        return ConfirmTransferSceneViewModel(
            request: request ?? ConfirmTransferRequest(data: data, simulation: simulation),
            wallet: wallet,
            service: GemConfirmTransferServiceMock(
                wallet: wallet,
                confirm: gemConfirmService,
                transactionState: transactionStateService,
            ),
            session: GemConfirmSessionMock(
                state: .mock(feeAsset: data.inputType.transactionAsset().map(), simulation: gemConfirmService.simulation, preload: nil),
                load: load,
            ),
            onComplete: onComplete,
        )
    }
}

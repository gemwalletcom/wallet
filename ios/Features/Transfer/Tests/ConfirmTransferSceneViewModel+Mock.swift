// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemConfirmData
import struct Gemstone.GemConfirmLoad
import enum Gemstone.GemExecuteResult
import struct Gemstone.GemTransferData
import GemstonePrimitives
import GemstonePrimitivesTestKit
import GemstoneServicesTestKit
import Primitives
import PrimitivesComponents
import StoreTestKit
@testable import Transfer
import TransferTestKit
import struct Gemstone.SimulationResult

@MainActor
extension ConfirmTransferSceneViewModel {
    static func mock(
        request: ConfirmTransferRequest? = nil,
        wallet: Wallet? = nil,
        data: GemTransferData = .mock(),
        simulation: SimulationResult? = nil,
        gemConfirmService: GemConfirmServiceMock = GemConfirmServiceMock(),
        load: Result<GemConfirmLoad, any Error>? = nil,
        execute: Result<GemExecuteResult, any Error> = .success(.signed(data: [])),
        session: GemConfirmSessionMock? = nil,
        onComplete: ((GemExecuteResult) -> Void)? = nil,
    ) -> ConfirmTransferSceneViewModel {
        let wallet = wallet ?? .mock(accounts: [.mock(chain: data.chain)])
        return ConfirmTransferSceneViewModel(
            request: request ?? ConfirmTransferRequest(data: data, simulation: simulation),
            wallet: wallet,
            session: session ?? GemConfirmSessionMock(
                state: .mock(transfer: data, feeAsset: data.feeAsset().map(), simulation: gemConfirmService.simulation, preload: nil),
                load: (load ?? .success(.mock())).map { GemConfirmLoad(transfer: data, sender: $0.sender, feeAsset: $0.feeAsset, metadata: $0.metadata, feeAssets: $0.feeAssets, simulation: $0.simulation, addressName: $0.addressName, preload: $0.preload) },
                execute: execute,
            ),
            onComplete: onComplete,
        )
    }
}

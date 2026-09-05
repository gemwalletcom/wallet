// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemConfirmSimulation
import struct Gemstone.GemConfirmSimulationState
import GemstonePrimitives
import Primitives
import PrimitivesComponents

struct ConfirmSimulationState {
    let result: SimulationResult?
    let warnings: [SimulationWarning]
    let hasCriticalWarning: Bool
    let payload: SimulationPayloadModel
    let headerData: AssetValueHeaderData?
    let balanceChanges: [SimulationAssetChange]

    init(
        result: SimulationResult?,
        warnings: [SimulationWarning],
        hasCriticalWarning: Bool,
        payload: SimulationPayloadModel,
        headerData: AssetValueHeaderData?,
        balanceChanges: [SimulationAssetChange],
    ) {
        self.result = result
        self.warnings = warnings
        self.hasCriticalWarning = hasCriticalWarning
        self.payload = payload
        self.headerData = headerData
        self.balanceChanges = balanceChanges
    }

    init(result: SimulationResult?, chain: Primitives.Chain) {
        self.init(
            result: result,
            warnings: result?.warnings ?? [],
            hasCriticalWarning: false,
            payload: SimulationPayloadModel(chain: chain, primaryFields: [], secondaryFields: []),
            headerData: nil,
            balanceChanges: [],
        )
    }

    init(_ state: GemConfirmSimulationState) throws {
        let details = state.simulation
        let simulation = try state.result.map { try Primitives.SimulationResult($0) }
        var payload = SimulationPayloadModel(
            chain: Primitives.Chain(core: state.chain),
            primaryFields: details?.primaryFields.map { $0.map() } ?? [],
            secondaryFields: details?.secondaryFields.map { $0.map() } ?? [],
        )
        payload.addressNames = state.names
        self.init(
            result: simulation,
            warnings: try state.warnings.map { try SimulationWarning($0) },
            hasCriticalWarning: details?.hasCriticalWarning ?? false,
            payload: payload,
            headerData: details?.header.flatMap {
                return AssetValueHeaderData(asset: $0.asset.map(), value: $0.value.map())
            },
            balanceChanges: details?.balanceChanges.map { SimulationAssetChange(asset: $0.asset.map(), value: $0.value) } ?? [],
        )
    }
}

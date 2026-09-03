// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemConfirmSimulation
import struct Gemstone.GemConfirmSimulationState
import GemstonePrimitives
import Primitives
import PrimitivesComponents
import struct Gemstone.GemTransferData

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

    init(
        data: GemTransferData,
        simulation: SimulationResult?,
        state: GemConfirmSimulationState,
    ) {
        let details = state.simulation
        let addressNames = state.names
        var payload = SimulationPayloadModel(
            chain: data.chain,
            primaryFields: details?.primaryFields.map { $0.map() } ?? [],
            secondaryFields: details?.secondaryFields.map { $0.map() } ?? [],
        )
        payload.addressNames = addressNames
        self.init(
            result: simulation,
            warnings: simulation?.warnings ?? [],
            hasCriticalWarning: details?.hasCriticalWarning ?? false,
            payload: payload,
            headerData: details?.header.flatMap { header in
                guard let value = try? header.value.map() else { return nil }
                return AssetValueHeaderData(asset: header.asset.map(), value: value)
            },
            balanceChanges: details?.balanceChanges.map { SimulationAssetChange(asset: $0.asset.map(), value: $0.value) } ?? [],
        )
    }
}

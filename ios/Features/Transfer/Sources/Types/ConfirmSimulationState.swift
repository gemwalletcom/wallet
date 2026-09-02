// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
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
            primaryFields: details?.primaryFields.compactMap { try? SimulationPayloadField($0) } ?? [],
            secondaryFields: details?.secondaryFields.compactMap { try? SimulationPayloadField($0) } ?? [],
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
            balanceChanges: details?.balanceChanges.compactMap { change in
                guard let value = BigInt(change.value, radix: 10) else { return nil }
                return SimulationAssetChange(asset: change.asset.map(), value: value)
            } ?? [],
        )
    }
}

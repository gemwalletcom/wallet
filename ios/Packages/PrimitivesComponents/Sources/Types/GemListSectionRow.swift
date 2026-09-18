// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import enum Gemstone.GemListRow
import struct Gemstone.GemListSection

public struct GemListSectionRow: Identifiable, Sendable {
    public let id: String
    public let row: GemListRow
}

public extension [GemListSection] {
    var listSections: [ListSection<GemListSectionRow>] {
        enumerated().map { index, section in
            ListSection(
                id: "\(index)",
                title: section.title.text,
                image: nil,
                values: section.rows.enumerated().map { row in
                    GemListSectionRow(id: "\(index)-\(row.offset)", row: row.element)
                },
                footer: section.footer.text,
            )
        }
    }
}

public extension ListSectionProvideable where Item == GemListSectionRow, ItemModel == GemListRow {
    func itemModel(for row: GemListSectionRow) -> any ItemModelProvidable<GemListRow> {
        row.row
    }
}

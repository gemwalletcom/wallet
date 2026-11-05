//
//  BigUIntTests.swift
//  BigInt
//
//  Created by Károly Lőrentey on 2015-12-27.
//  Copyright © 2016-2017 Károly Lőrentey.
//

import XCTest
import Foundation
@testable import BigInt

extension BigUInt.Kind: Equatable {
    public static func ==(left: BigUInt.Kind, right: BigUInt.Kind) -> Bool {
        switch (left, right) {
        case let (.inline(l0, l1), .inline(r0, r1)): return l0 == r0 && l1 == r1
        case let (.slice(from: ls, to: le), .slice(from: rs, to: re)): return ls == rs && le == re
        case (.array, .array): return true
        default: return false
        }
    }
}

class BigUIntTests: XCTestCase {
    typealias Word = BigUInt.Word

    func check(_ value: BigUInt, _ kind: BigUInt.Kind?, _ words: [Word], file: StaticString = #filePath, line: UInt = #line) {
        if let kind = kind {
            XCTAssertEqual(
                value.kind, kind,
                "Mismatching kind: \(value.kind) vs. \(kind)",
                file: file, line: line)
        }
        XCTAssertEqual(
            Array(value.words), words,
            "Mismatching words: \(value.words) vs. \(words)",
            file: file, line: line)
        XCTAssertEqual(
            value.isZero, words.isEmpty,
            "Mismatching isZero: \(value.isZero) vs. \(words.isEmpty)",
            file: file, line: line)
        XCTAssertEqual(
            value.count, words.count,
            "Mismatching count: \(value.count) vs. \(words.count)",
            file: file, line: line)
        for i in 0 ..< words.count {
            XCTAssertEqual(
                value[i], words[i],
                "Mismatching word at index \(i): \(value[i]) vs. \(words[i])",
                file: file, line: line)
        }
        for i in words.count ..< words.count + 10 {
            XCTAssertEqual(
                value[i], 0,
                "Expected 0 word at index \(i), got \(value[i])",
                file: file, line: line)
        }
    }

    func check(_ value: BigUInt?, _ kind: BigUInt.Kind?, _ words: [Word], file: StaticString = #filePath, line: UInt = #line) {
        guard let value = value else {
            XCTFail("Expected non-nil BigUInt", file: file, line: line)
            return
        }
        check(value, kind, words, file: file, line: line)
    }

    func testInit_WordBased() {
        check(BigUInt(), .inline(0, 0), [])

        check(BigUInt(word: 0), .inline(0, 0), [])
        check(BigUInt(word: 1), .inline(1, 0), [1])
        check(BigUInt(word: Word.max), .inline(Word.max, 0), [Word.max])

        check(BigUInt(low: 0, high: 0), .inline(0, 0), [])
        check(BigUInt(low: 0, high: 1), .inline(0, 1), [0, 1])
        check(BigUInt(low: 1, high: 0), .inline(1, 0), [1])
        check(BigUInt(low: 1, high: 2), .inline(1, 2), [1, 2])

        check(BigUInt(words: []), .array, [])
        check(BigUInt(words: [0, 0, 0, 0]), .array, [])
        check(BigUInt(words: [1]), .array, [1])
        check(BigUInt(words: [1, 2, 3, 0, 0]), .array, [1, 2, 3])
        check(BigUInt(words: [0, 1, 2, 3, 4]), .array, [0, 1, 2, 3, 4])

        check(BigUInt(words: [], from: 0, to: 0), .inline(0, 0), [])
        check(BigUInt(words: [1, 2, 3, 4], from: 0, to: 4), .array, [1, 2, 3, 4])
        check(BigUInt(words: [1, 2, 3, 4], from: 0, to: 3), .slice(from: 0, to: 3), [1, 2, 3])
        check(BigUInt(words: [1, 2, 3, 4], from: 1, to: 4), .slice(from: 1, to: 4), [2, 3, 4])
        check(BigUInt(words: [1, 2, 3, 4], from: 0, to: 2), .inline(1, 2), [1, 2])
        check(BigUInt(words: [1, 2, 3, 4], from: 0, to: 1), .inline(1, 0), [1])
        check(BigUInt(words: [1, 2, 3, 4], from: 1, to: 1), .inline(0, 0), [])
        check(BigUInt(words: [0, 0, 0, 1, 0, 0, 0, 2], from: 0, to: 4), .slice(from: 0, to: 4), [0, 0, 0, 1])
        check(BigUInt(words: [0, 0, 0, 1, 0, 0, 0, 2], from: 0, to: 3), .inline(0, 0), [])
        check(BigUInt(words: [0, 0, 0, 1, 0, 0, 0, 2], from: 2, to: 6), .inline(0, 1), [0, 1])

        check(BigUInt(words: [].lazy), .inline(0, 0), [])
        check(BigUInt(words: [1].lazy), .inline(1, 0), [1])
        check(BigUInt(words: [1, 2].lazy), .inline(1, 2), [1, 2])
        check(BigUInt(words: [1, 2, 3].lazy), .array, [1, 2, 3])
        check(BigUInt(words: [1, 2, 3, 0, 0, 0, 0].lazy), .array, [1, 2, 3])

        check(BigUInt(words: IteratorSequence([].makeIterator())), .inline(0, 0), [])
        check(BigUInt(words: IteratorSequence([1].makeIterator())), .inline(1, 0), [1])
        check(BigUInt(words: IteratorSequence([1, 2].makeIterator())), .inline(1, 2), [1, 2])
        check(BigUInt(words: IteratorSequence([1, 2, 3].makeIterator())), .array, [1, 2, 3])
        check(BigUInt(words: IteratorSequence([1, 2, 3, 0, 0, 0, 0].makeIterator())), .array, [1, 2, 3])
    }

    func testInit_BinaryInteger() {
        XCTAssertNil(BigUInt(exactly: -42))
        check(BigUInt(exactly: 0 as Int), .inline(0, 0), [])
        check(BigUInt(exactly: 42 as Int), .inline(42, 0), [42])
        check(BigUInt(exactly: 43 as UInt), .inline(43, 0), [43])
        check(BigUInt(exactly: 44 as UInt8), .inline(44, 0), [44])
        check(BigUInt(exactly: BigUInt(words: [])), .inline(0, 0), [])
        check(BigUInt(exactly: BigUInt(words: [1])), .inline(1, 0), [1])
        check(BigUInt(exactly: BigUInt(words: [1, 2])), .inline(1, 2), [1, 2])
        check(BigUInt(exactly: BigUInt(words: [1, 2, 3, 4])), .array, [1, 2, 3, 4])
    }

    func testInit_FloatingPoint() {
        check(BigUInt(exactly: -0.0 as Float), nil, [])
        check(BigUInt(exactly: -0.0 as Double), nil, [])

        XCTAssertNil(BigUInt(exactly: -42.0 as Float))
        XCTAssertNil(BigUInt(exactly: -42.0 as Double))

        XCTAssertNil(BigUInt(exactly: 42.5 as Float))
        XCTAssertNil(BigUInt(exactly: 42.5 as Double))

        check(BigUInt(exactly: 100 as Float), nil, [100])
        check(BigUInt(exactly: 100 as Double), nil, [100])

        check(BigUInt(exactly: Float.greatestFiniteMagnitude), nil,
              convertWords([0, 0xFFFFFF0000000000]))

        check(BigUInt(exactly: Double.greatestFiniteMagnitude), nil,
              convertWords([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xFFFFFFFFFFFFF800]))

        XCTAssertNil(BigUInt(exactly: Float.leastNormalMagnitude))
        XCTAssertNil(BigUInt(exactly: Double.leastNormalMagnitude))

        XCTAssertNil(BigUInt(exactly: Float.infinity))
        XCTAssertNil(BigUInt(exactly: Double.infinity))

        XCTAssertNil(BigUInt(exactly: Float.nan))
        XCTAssertNil(BigUInt(exactly: Double.nan))

        check(BigUInt(0 as Float), nil, [])
        check(BigUInt(Float.leastNonzeroMagnitude), nil, [])
        check(BigUInt(Float.leastNormalMagnitude), nil, [])
        check(BigUInt(0.5 as Float), nil, [])
        check(BigUInt(1.5 as Float), nil, [1])
        check(BigUInt(42 as Float), nil, [42])
        check(BigUInt(Double(sign: .plus, exponent: 2 * Word.bitWidth, significand: 1.0)),
              nil, [0, 0, 1])
    }

    func testInit_Buffer() {
        func test(_ b: BigUInt, _ d: Array<UInt8>, file: StaticString = #filePath, line: UInt = #line) {
            d.withUnsafeBytes { buffer in
                let initialized = BigUInt(buffer)
                XCTAssertEqual(initialized, b, file: file, line: line)
            }
        }
        
        // Positive integers
        test(BigUInt(), [])
        test(BigUInt(1), [0x01])
        test(BigUInt(2), [0x02])
        test(BigUInt(0x0102030405060708), [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08])
        test(BigUInt(0x01) << 64 + BigUInt(0x0203040506070809), [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 09])
    }

    func testConversionToFloatingPoint() {
        func test<F: BinaryFloatingPoint>(_ a: BigUInt, _ b: F, file: StaticString = #filePath, line: UInt = #line)
        where F.RawExponent: FixedWidthInteger, F.RawSignificand: FixedWidthInteger {
            let f = F(a)
            XCTAssertEqual(f, b, file: file, line: line)
        }

        for i in 0 ..< 100 {
            test(BigUInt(i), Double(i))
        }
        test(BigUInt(0x5A5A5A), 0x5A5A5A as Double)
        test(BigUInt(1) << 64, 0x1p64 as Double)
        test(BigUInt(0x5A5A5A) << 64, 0x5A5A5Ap64 as Double)
        test(BigUInt(1) << 1023, 0x1p1023 as Double)
        test(BigUInt(10) << 1020, 0xAp1020 as Double)
        test(BigUInt(1) << 1024, Double.infinity)
        test(BigUInt(words: convertWords([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xFFFFFFFFFFFFF800])),
             Double.greatestFiniteMagnitude)
        test(BigUInt(UInt64.max), 0x1p64 as Double)

        for i in 0 ..< 100 {
            test(BigUInt(i), Float(i))
        }
        test(BigUInt(0x5A5A5A), 0x5A5A5A as Float)
        test(BigUInt(1) << 64, 0x1p64 as Float)
        test(BigUInt(0x5A5A5A) << 64, 0x5A5A5Ap64 as Float)
        test(BigUInt(1) << 1023, 0x1p1023 as Float)
        test(BigUInt(10) << 1020, 0xAp1020 as Float)
        test(BigUInt(1) << 1024, Float.infinity)
        test(BigUInt(words: convertWords([0, 0xFFFFFF0000000000])),
             Float.greatestFiniteMagnitude)

        // Test rounding
        test(BigUInt(0xFFFFFF0000000000 as UInt64), 0xFFFFFFp40 as Float)
        test(BigUInt(0xFFFFFF7FFFFFFFFF as UInt64), 0xFFFFFFp40 as Float)
        test(BigUInt(0xFFFFFF8000000000 as UInt64), 0x1p64 as Float)
        test(BigUInt(0xFFFFFFFFFFFFFFFF as UInt64), 0x1p64 as Float)

        test(BigUInt(0xFFFFFE0000000000 as UInt64), 0xFFFFFEp40 as Float)
        test(BigUInt(0xFFFFFE7FFFFFFFFF as UInt64), 0xFFFFFEp40 as Float)
        test(BigUInt(0xFFFFFE8000000000 as UInt64), 0xFFFFFEp40 as Float)
        test(BigUInt(0xFFFFFEFFFFFFFFFF as UInt64), 0xFFFFFFp40 as Float)

        test(BigUInt(0x8000010000000000 as UInt64), 0x800001p40 as Float)
        test(BigUInt(0x8000017FFFFFFFFF as UInt64), 0x800001p40 as Float)
        test(BigUInt(0x8000018000000000 as UInt64), 0x800002p40 as Float)
        test(BigUInt(0x800001FFFFFFFFFF as UInt64), 0x800002p40 as Float)

        test(BigUInt(0x8000020000000000 as UInt64), 0x800002p40 as Float)
        test(BigUInt(0x8000027FFFFFFFFF as UInt64), 0x800002p40 as Float)
        test(BigUInt(0x8000028000000000 as UInt64), 0x800002p40 as Float)
        test(BigUInt(0x800002FFFFFFFFFF as UInt64), 0x800003p40 as Float)

    }

    func testInit_Misc() {
        check(BigUInt(0), .inline(0, 0), [])
        check(BigUInt(42), .inline(42, 0), [42])
        check(BigUInt(BigUInt(words: [1, 2, 3])), .array, [1, 2, 3])

        check(BigUInt(truncatingIfNeeded: 0 as Int8), .inline(0, 0), [])
        check(BigUInt(truncatingIfNeeded: 1 as Int8), .inline(1, 0), [1])
        check(BigUInt(truncatingIfNeeded: -1 as Int8), .inline(Word.max, 0), [Word.max])
        check(BigUInt(truncatingIfNeeded: BigUInt(words: [1, 2, 3])), .array, [1, 2, 3])

        check(BigUInt(clamping: 0), .inline(0, 0), [])
        check(BigUInt(clamping: -100), .inline(0, 0), [])
        check(BigUInt(clamping: Word.max), .inline(Word.max, 0), [Word.max])
    }

    func testEnsureArray() {
        var a = BigUInt()
        a.ensureArray()
        check(a, .array, [])

        a = BigUInt(word: 1)
        a.ensureArray()
        check(a, .array, [1])

        a = BigUInt(low: 1, high: 2)
        a.ensureArray()
        check(a, .array, [1, 2])

        a = BigUInt(words: [1, 2, 3, 4])
        a.ensureArray()
        check(a, .array, [1, 2, 3, 4])

        a = BigUInt(words: [1, 2, 3, 4, 5, 6], from: 1, to: 5)
        a.ensureArray()
        check(a, .array, [2, 3, 4, 5])
    }

    func testCapacity() {
        XCTAssertEqual(BigUInt(low: 1, high: 2).capacity, 0)
        XCTAssertEqual(BigUInt(words: 1 ..< 10).extract(2 ..< 5).capacity, 0)
        var words: [Word] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]
        words.reserveCapacity(100)
        XCTAssertGreaterThanOrEqual(BigUInt(words: words).capacity, 100)
    }

    func testReserveCapacity() {
        var a = BigUInt()
        a.reserveCapacity(100)
        check(a, .array, [])
        XCTAssertGreaterThanOrEqual(a.capacity, 100)

        a = BigUInt(word: 1)
        a.reserveCapacity(100)
        check(a, .array, [1])
        XCTAssertGreaterThanOrEqual(a.capacity, 100)

        a = BigUInt(low: 1, high: 2)
        a.reserveCapacity(100)
        check(a, .array, [1, 2])
        XCTAssertGreaterThanOrEqual(a.capacity, 100)

        a = BigUInt(words: [1, 2, 3, 4])
        a.reserveCapacity(100)
        check(a, .array, [1, 2, 3, 4])
        XCTAssertGreaterThanOrEqual(a.capacity, 100)

        a = BigUInt(words: [1, 2, 3, 4, 5, 6], from: 1, to: 5)
        a.reserveCapacity(100)
        check(a, .array, [2, 3, 4, 5])
        XCTAssertGreaterThanOrEqual(a.capacity, 100)
    }

    func testLoad() {
        var a: BigUInt = 0
        a.reserveCapacity(100)

        a.load(BigUInt(low: 1, high: 2))
        check(a, .array, [1, 2])
        XCTAssertGreaterThanOrEqual(a.capacity, 100)

        a.load(BigUInt(words: [1, 2, 3, 4, 5, 6]))
        check(a, .array, [1, 2, 3, 4, 5, 6])
        XCTAssertGreaterThanOrEqual(a.capacity, 100)

        a.clear()
        check(a, .array, [])
        XCTAssertGreaterThanOrEqual(a.capacity, 100)
    }

    func testInitFromLiterals() {
        check(0, .inline(0, 0), [])
        check(42, .inline(42, 0), [42])
        check("42", .inline(42, 0), [42])

        check("1512366075204170947332355369683137040",
              .inline(0xFEDCBA9876543210, 0x0123456789ABCDEF),
              [0xFEDCBA9876543210, 0x0123456789ABCDEF])

        // I have no idea how to exercise these in the wild
        check(BigUInt(unicodeScalarLiteral: UnicodeScalar(52)), .inline(4, 0), [4])
        check(BigUInt(extendedGraphemeClusterLiteral: "4"), .inline(4, 0), [4])
    }

    func testSubscriptingGetter() {
        let a = BigUInt(words: [1, 2])
        XCTAssertEqual(a[0], 1)
        XCTAssertEqual(a[1], 2)
        XCTAssertEqual(a[2], 0)
        XCTAssertEqual(a[3], 0)
        XCTAssertEqual(a[10000], 0)

        let b = BigUInt(low: 1, high: 2)
        XCTAssertEqual(b[0], 1)
        XCTAssertEqual(b[1], 2)
        XCTAssertEqual(b[2], 0)
        XCTAssertEqual(b[3], 0)
        XCTAssertEqual(b[10000], 0)
    }

    func testSubscriptingSetter() {
        var a = BigUInt()

        check(a, .inline(0, 0), [])
        a[10] = 0
        check(a, .inline(0, 0), [])
        a[0] = 42
        check(a, .inline(42, 0), [42])
        a[10] = 23
        check(a, .array, [42, 0, 0, 0, 0, 0, 0, 0, 0, 0, 23])
        a[0] = 0
        check(a, .array, [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 23])
        a[10] = 0
        check(a, .array, [])

        a = BigUInt(words: [0, 1, 2, 3, 4, 5, 6], from: 1, to: 5)
        a[2] = 42
        check(a, .array, [1, 2, 42, 4])
    }

    func testSlice() {
        let a = BigUInt(words: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10])
        check(a.extract(3 ..< 6), .slice(from: 3, to: 6), [3, 4, 5])
        check(a.extract(3 ..< 5), .inline(3, 4), [3, 4])
        check(a.extract(3 ..< 4), .inline(3, 0), [3])
        check(a.extract(3 ..< 3), .inline(0, 0), [])
        check(a.extract(0 ..< 100), .array, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10])
        check(a.extract(100 ..< 200), .inline(0, 0), [])

        let b = BigUInt(low: 1, high: 2)
        check(b.extract(0 ..< 2), .inline(1, 2), [1, 2])
        check(b.extract(0 ..< 1), .inline(1, 0), [1])
        check(b.extract(1 ..< 2), .inline(2, 0), [2])
        check(b.extract(1 ..< 1), .inline(0, 0), [])
        check(b.extract(0 ..< 100), .inline(1, 2), [1, 2])
        check(b.extract(100 ..< 200), .inline(0, 0), [])

        let c = BigUInt(words: [1, 0, 0, 0, 2, 0, 0, 0, 3, 4, 5, 0, 0, 6, 0, 0, 0, 7])
        check(c.extract(0 ..< 4), .inline(1, 0), [1])
        check(c.extract(1 ..< 5), .slice(from: 1, to: 5), [0, 0, 0, 2])
        check(c.extract(1 ..< 8), .slice(from: 1, to: 5), [0, 0, 0, 2])
        check(c.extract(6 ..< 12), .slice(from: 6, to: 11), [0, 0, 3, 4, 5])
        check(c.extract(4 ..< 7), .inline(2, 0), [2])

        let d = c.extract(3 ..< 14)
                                        // 0  1  2  3  4  5  6  7  8  9 10
        check(d, .slice(from: 3, to: 14), [0, 2, 0, 0, 0, 3, 4, 5, 0, 0, 6])
        check(d.extract(1 ..< 5), .inline(2, 0), [2])
        check(d.extract(0 ..< 3), .inline(0, 2), [0, 2])
        check(d.extract(1 ..< 6), .slice(from: 4, to: 9), [2, 0, 0, 0, 3])
        check(d.extract(7 ..< 1000), .slice(from: 10, to: 14), [5, 0, 0, 6])
        check(d.extract(10 ..< 1000), .inline(6, 0), [6])
        check(d.extract(11 ..< 1000), .inline(0, 0), [])
    }

    func testSigns() {
        XCTAssertFalse(BigUInt.isSigned)

        XCTAssertEqual(BigUInt().signum(), 0)
        XCTAssertEqual(BigUInt(words: []).signum(), 0)
        XCTAssertEqual(BigUInt(words: [0, 1, 2]).signum(), 1)
        XCTAssertEqual(BigUInt(word: 42).signum(), 1)
    }

    func testBits() {
        let indices: Set<Int> = [0, 13, 59, 64, 79, 130]
        var value: BigUInt = 0
        for i in indices {
            value[bitAt: i] = true
        }
        for i in 0 ..< 300 {
            XCTAssertEqual(value[bitAt: i], indices.contains(i))
        }
        check(value, nil, convertWords([0x0800000000002001, 0x8001, 0x04]))
        for i in indices {
            value[bitAt: i] = false
        }
        check(value, nil, [])
    }

    func testStrideableRequirements() {
        XCTAssertEqual(BigUInt(10), BigUInt(4).advanced(by: BigInt(6)))
        XCTAssertEqual(BigUInt(4), BigUInt(10).advanced(by: BigInt(-6)))
        XCTAssertEqual(BigInt(6), BigUInt(4).distance(to: 10))
        XCTAssertEqual(BigInt(-6), BigUInt(10).distance(to: 4))
    }

    func testRightShift_ByWord() {
        var a = BigUInt()
        a.shiftRight(byWords: 1)
        check(a, .inline(0, 0), [])

        a = BigUInt(low: 1, high: 2)
        a.shiftRight(byWords: 0)
        check(a, .inline(1, 2), [1, 2])

        a = BigUInt(low: 1, high: 2)
        a.shiftRight(byWords: 1)
        check(a, .inline(2, 0), [2])

        a = BigUInt(low: 1, high: 2)
        a.shiftRight(byWords: 2)
        check(a, .inline(0, 0), [])

        a = BigUInt(low: 1, high: 2)
        a.shiftRight(byWords: 10)
        check(a, .inline(0, 0), [])


        a = BigUInt(words: [0, 1, 2, 3, 4])
        a.shiftRight(byWords: 1)
        check(a, .array, [1, 2, 3, 4])

        a = BigUInt(words: [0, 1, 2, 3, 4])
        a.shiftRight(byWords: 2)
        check(a, .array, [2, 3, 4])

        a = BigUInt(words: [0, 1, 2, 3, 4])
        a.shiftRight(byWords: 5)
        check(a, .array, [])

        a = BigUInt(words: [0, 1, 2, 3, 4])
        a.shiftRight(byWords: 100)
        check(a, .array, [])


        a = BigUInt(words: [0, 1, 2, 3, 4, 5, 6], from: 1, to: 6)
        check(a, .slice(from: 1, to: 6), [1, 2, 3, 4, 5])
        a.shiftRight(byWords: 1)
        check(a, .slice(from: 2, to: 6), [2, 3, 4, 5])

        a = BigUInt(words: [0, 1, 2, 3, 4, 5, 6], from: 1, to: 6)
        a.shiftRight(byWords: 2)
        check(a, .slice(from: 3, to: 6), [3, 4, 5])

        a = BigUInt(words: [0, 1, 2, 3, 4, 5, 6], from: 1, to: 6)
        a.shiftRight(byWords: 3)
        check(a, .inline(4, 5), [4, 5])

        a = BigUInt(words: [0, 1, 2, 3, 4, 5, 6], from: 1, to: 6)
        a.shiftRight(byWords: 4)
        check(a, .inline(5, 0), [5])

        a = BigUInt(words: [0, 1, 2, 3, 4, 5, 6], from: 1, to: 6)
        a.shiftRight(byWords: 5)
        check(a, .inline(0, 0), [])

        a = BigUInt(words: [0, 1, 2, 3, 4, 5, 6], from: 1, to: 6)
        a.shiftRight(byWords: 10)
        check(a, .inline(0, 0), [])
    }

    func testLeftShift_ByWord() {
        var a = BigUInt()
        a.shiftLeft(byWords: 1)
        check(a, .inline(0, 0), [])

        a = BigUInt(word: 1)
        a.shiftLeft(byWords: 0)
        check(a, .inline(1, 0), [1])

        a = BigUInt(word: 1)
        a.shiftLeft(byWords: 1)
        check(a, .inline(0, 1), [0, 1])

        a = BigUInt(word: 1)
        a.shiftLeft(byWords: 2)
        check(a, .array, [0, 0, 1])

        a = BigUInt(low: 1, high: 2)
        a.shiftLeft(byWords: 1)
        check(a, .array, [0, 1, 2])

        a = BigUInt(low: 1, high: 2)
        a.shiftLeft(byWords: 2)
        check(a, .array, [0, 0, 1, 2])

        a = BigUInt(words: [1, 2, 3, 4, 5, 6])
        a.shiftLeft(byWords: 1)
        check(a, .array, [0, 1, 2, 3, 4, 5, 6])

        a = BigUInt(words: [1, 2, 3, 4, 5, 6])
        a.shiftLeft(byWords: 10)
        check(a, .array, [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 2, 3, 4, 5, 6])

        a = BigUInt(words: [0, 1, 2, 3, 4, 5, 6], from: 2, to: 6)
        a.shiftLeft(byWords: 1)
        check(a, .array, [0, 2, 3, 4, 5])

        a = BigUInt(words: [0, 1, 2, 3, 4, 5, 6], from: 2, to: 6)
        a.shiftLeft(byWords: 3)
        check(a, .array, [0, 0, 0, 2, 3, 4, 5])
    }

    func testSplit() {
        let a = BigUInt(words: [0, 1, 2, 3])
        XCTAssertEqual(a.split.low, BigUInt(words: [0, 1]))
        XCTAssertEqual(a.split.high, BigUInt(words: [2, 3]))
    }

    func testLowHigh() {
        let a = BigUInt(words: [0, 1, 2, 3])
        check(a.low, .inline(0, 1), [0, 1])
        check(a.high, .inline(2, 3), [2, 3])
        check(a.low.low, .inline(0, 0), [])
        check(a.low.high, .inline(1, 0), [1])
        check(a.high.low, .inline(2, 0), [2])
        check(a.high.high, .inline(3, 0), [3])

        let b = BigUInt(words: [0, 1, 2, 3, 4, 5])

        let bl = b.low
        check(bl, .slice(from: 0, to: 3), [0, 1, 2])
        let bh = b.high
        check(bh, .slice(from: 3, to: 6), [3, 4, 5])

        let bll = bl.low
        check(bll, .inline(0, 1), [0, 1])
        let blh = bl.high
        check(blh, .inline(2, 0), [2])
        let bhl = bh.low
        check(bhl, .inline(3, 4), [3, 4])
        let bhh = bh.high
        check(bhh, .inline(5, 0), [5])

        let blhl = bll.low
        check(blhl, .inline(0, 0), [])
        let blhh = bll.high
        check(blhh, .inline(1, 0), [1])
        let bhhl = bhl.low
        check(bhhl, .inline(3, 0), [3])
        let bhhh = bhl.high
        check(bhhh, .inline(4, 0), [4])
    }

    func testComparison() {
        XCTAssertEqual(BigUInt(words: [1, 2, 3]), BigUInt(words: [1, 2, 3]))
        XCTAssertNotEqual(BigUInt(words: [1, 2]), BigUInt(words: [1, 2, 3]))
        XCTAssertNotEqual(BigUInt(words: [1, 2, 3]), BigUInt(words: [1, 3, 3]))
        XCTAssertEqual(BigUInt(words: [1, 2, 3, 4, 5, 6]).low.high, BigUInt(words: [3]))

        XCTAssertTrue(BigUInt(words: [1, 2]) < BigUInt(words: [1, 2, 3]))
        XCTAssertTrue(BigUInt(words: [1, 2, 2]) < BigUInt(words: [1, 2, 3]))
        XCTAssertFalse(BigUInt(words: [1, 2, 3]) < BigUInt(words: [1, 2, 3]))
        XCTAssertTrue(BigUInt(words: [3, 3]) < BigUInt(words: [1, 2, 3, 4, 5, 6]).extract(2 ..< 4))
        XCTAssertTrue(BigUInt(words: [1, 2, 3, 4, 5, 6]).low.high < BigUInt(words: [3, 5]))
    }

    func testHashing() {
        var hashes: [Int] = []
        hashes.append(BigUInt(words: []).hashValue)
        hashes.append(BigUInt(words: [1]).hashValue)
        hashes.append(BigUInt(words: [2]).hashValue)
        hashes.append(BigUInt(words: [0, 1]).hashValue)
        hashes.append(BigUInt(words: [1, 1]).hashValue)
        hashes.append(BigUInt(words: [1, 2]).hashValue)
        hashes.append(BigUInt(words: [2, 1]).hashValue)
        hashes.append(BigUInt(words: [2, 2]).hashValue)
        hashes.append(BigUInt(words: [1, 2, 3, 4, 5]).hashValue)
        hashes.append(BigUInt(words: [5, 4, 3, 2, 1]).hashValue)
        hashes.append(BigUInt(words: [Word.max]).hashValue)
        hashes.append(BigUInt(words: [Word.max, Word.max]).hashValue)
        hashes.append(BigUInt(words: [Word.max, Word.max, Word.max]).hashValue)
        hashes.append(BigUInt(words: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]).hashValue)
        XCTAssertEqual(hashes.count, Set(hashes).count)
    }

    func checkData(_ bytes: [UInt8], _ value: BigUInt, file: StaticString = #filePath, line: UInt = #line) {
        XCTAssertEqual(BigUInt(Data(bytes)), value, file: file, line: line)
        XCTAssertEqual(bytes.withUnsafeBytes { buffer in BigUInt(buffer) }, value, file: file, line: line)
    }

    func testConversionFromBytes() {
        checkData([], 0)
        checkData([0], 0)
        checkData([0, 0, 0, 0, 0, 0, 0, 0], 0)
        checkData([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], 0)
        checkData([1], 1)
        checkData([2], 2)
        checkData([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1], 1)
        checkData([0x01, 0x02, 0x03, 0x04, 0x05], 0x0102030405)
        checkData([0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08], 0x0102030405060708)
        checkData([0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A],
                  BigUInt(0x0102) << 64 + BigUInt(0x030405060708090A))
        checkData([0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
                  BigUInt(1) << 80)
        checkData([0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10],
                  BigUInt(0x0102030405060708) << 64 + BigUInt(0x090A0B0C0D0E0F10))
        checkData([0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10, 0x11],
                  ((BigUInt(1) << 128) as BigUInt) + BigUInt(0x0203040506070809) << 64 + BigUInt(0x0A0B0C0D0E0F1011))
    }

    func testConversionToData() {
        func test(_ b: BigUInt, _ d: Array<UInt8>, file: StaticString = #filePath, line: UInt = #line) {
            let expected = Data(d)
            let actual = b.serialize()
            XCTAssertEqual(actual, expected, file: file, line: line)
            XCTAssertEqual(BigUInt(actual), b, file: file, line: line)
        }

        test(BigUInt(), [])
        test(BigUInt(1), [0x01])
        test(BigUInt(2), [0x02])
        test(BigUInt(0x010203), [0x1, 0x2, 0x3])
        test(BigUInt(0x0102030405060708), [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08])
        test(BigUInt(0x01) << 64 + BigUInt(0x0203040506070809), [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09])
    }

    func testCodable() {
        func test(_ a: BigUInt, file: StaticString = #filePath, line: UInt = #line) {
            do {
                let json = try JSONEncoder().encode(a)
                print(String(data: json, encoding: .utf8)!)
                let b = try JSONDecoder().decode(BigUInt.self, from: json)
                XCTAssertEqual(a, b, file: file, line: line)
            }
            catch let error {
                XCTFail("Error thrown: \(error.localizedDescription)", file: file, line: line)
            }
        }
        test(0)
        test(1)
        test(0x0102030405060708)
        test(BigUInt(1) << 64)
        test(BigUInt(words: [1, 2, 3, 4, 5, 6, 7]))

        XCTAssertThrowsError(try JSONDecoder().decode(BigUInt.self, from: "[\"*\", 1]".data(using: .utf8)!)) { error in
            guard let error = error as? DecodingError else { XCTFail("Expected a decoding error"); return }
            guard case .dataCorrupted(let context) = error else { XCTFail("Expected a dataCorrupted error"); return }
            XCTAssertEqual(context.debugDescription, "Invalid big integer sign")
        }
        XCTAssertThrowsError(try JSONDecoder().decode(BigUInt.self, from: "[\"-\", 1]".data(using: .utf8)!)) { error in
            guard let error = error as? DecodingError else { XCTFail("Expected a decoding error"); return }
            guard case .dataCorrupted(let context) = error else { XCTFail("Expected a dataCorrupted error"); return }
            XCTAssertEqual(context.debugDescription, "BigUInt cannot hold a negative value")
        }
    }

    func testAddition() {
        XCTAssertEqual(BigUInt(0) + BigUInt(0), BigUInt(0))
        XCTAssertEqual(BigUInt(0) + BigUInt(Word.max), BigUInt(Word.max))
        XCTAssertEqual(BigUInt(Word.max) + BigUInt(1), BigUInt(words: [0, 1]))

        check(BigUInt(3) + BigUInt(42), .inline(45, 0), [45])
        check(BigUInt(3) + BigUInt(42), .inline(45, 0), [45])

        check(0 + BigUInt(Word.max), .inline(Word.max, 0), [Word.max])
        check(1 + BigUInt(Word.max), .inline(0, 1), [0, 1])
        check(BigUInt(low: 0, high: 1) + BigUInt(low: 3, high: 4), .inline(3, 5), [3, 5])
        check(BigUInt(low: 3, high: 5) + BigUInt(low: 0, high: Word.max), .array, [3, 4, 1])
        check(BigUInt(words: [3, 4, 1]) + BigUInt(low: 0, high: Word.max), .array, [3, 3, 2])
        check(BigUInt(words: [3, 3, 2]) + 2, .array, [5, 3, 2])
        check(BigUInt(words: [Word.max - 5, Word.max, 4, Word.max]).addingWord(6), .array, [0, 0, 5, Word.max])

        var b = BigUInt(words: [Word.max, 2, Word.max])
        b.increment()
        check(b, .array, [0, 3, Word.max])
    }

    func testShiftedAddition() {
        var b = BigUInt()
        b.add(1, shiftedBy: 1)
        check(b, .inline(0, 1), [0, 1])

        b.add(2, shiftedBy: 3)
        check(b, .array, [0, 1, 0, 2])

        b.add(BigUInt(Word.max), shiftedBy: 1)
        check(b, .array, [0, 0, 1, 2])
    }

    func testSubtraction() {
        var a1 = BigUInt(words: [1, 2, 3, 4])
        XCTAssertEqual(false, a1.subtractWordReportingOverflow(3, shiftedBy: 1))
        check(a1, .array, [1, Word.max, 2, 4])

        let (diff, overflow) = BigUInt(words: [1, 2, 3, 4]).subtractingWordReportingOverflow(2)
        XCTAssertEqual(false, overflow)
        check(diff, .array, [Word.max, 1, 3, 4])

        var a2 = BigUInt(words: [1, 2, 3, 4])
        XCTAssertEqual(true, a2.subtractWordReportingOverflow(5, shiftedBy: 3))
        check(a2, .array, [1, 2, 3, Word.max])

        var a3 = BigUInt(words: [1, 2, 3, 4])
        a3.subtractWord(4, shiftedBy: 3)
        check(a3, .array, [1, 2, 3])

        var a4 = BigUInt(words: [1, 2, 3, 4])
        a4.decrement()
        check(a4, .array, [0, 2, 3, 4])
        a4.decrement()
        check(a4, .array, [Word.max, 1, 3, 4])

        check(BigUInt(words: [1, 2, 3, 4]).subtractingWord(5),
              .array, [Word.max - 3, 1, 3, 4])

        check(BigUInt(0) - BigUInt(0), .inline(0, 0), [])

        var b = BigUInt(words: [1, 2, 3, 4])
        XCTAssertEqual(false, b.subtractReportingOverflow(BigUInt(words: [0, 1, 1, 1])))
        check(b, .array, [1, 1, 2, 3])

        let b1 = BigUInt(words: [1, 1, 2, 3]).subtractingReportingOverflow(BigUInt(words: [1, 1, 3, 3]))
        XCTAssertEqual(true, b1.overflow)
        check(b1.partialValue, .array, [0, 0, Word.max, Word.max])

        let b2 = BigUInt(words: [0, 0, 1]) - BigUInt(words: [1])
        check(b2, .array, [Word.max, Word.max])

        var b3 = BigUInt(words: [1, 0, 0, 1])
        b3 -= 2
        check(b3, .array, [Word.max, Word.max, Word.max])

        check(BigUInt(42) - BigUInt(23), .inline(19, 0), [19])
    }

    func testMultiplyByWord() {
        check(BigUInt(words: [1, 2, 3, 4]).multiplied(byWord: 0), .inline(0, 0), [])
        check(BigUInt(words: [1, 2, 3, 4]).multiplied(byWord: 2), .array, [2, 4, 6, 8])

        let full = Word.max

        check(BigUInt(words: [full, 0, full, 0, full]).multiplied(byWord: 2),
              .array, [full - 1, 1, full - 1, 1, full - 1, 1])

        check(BigUInt(words: [full, full, full]).multiplied(byWord: 2),
              .array, [full - 1, full, full, 1])

        check(BigUInt(words: [full, full, full]).multiplied(byWord: full),
              .array, [1, full, full, full - 1])

        check(BigUInt("11111111111111111111111111111111", radix: 16)!.multiplied(byWord: 15),
              .array, convertWords([UInt64.max, UInt64.max]))

        check(BigUInt("11111111111111111111111111111112", radix: 16)!.multiplied(byWord: 15),
              .array, convertWords([0xE, 0, 0x1]))

        check(BigUInt(low: 1, high: 2).multiplied(byWord: 3), .inline(3, 6), [3, 6])
    }

    func testMultiplication() {
        func test() {
            check(BigUInt(low: 1, high: 1) * BigUInt(word: 3), .inline(3, 3), [3, 3])
            check(BigUInt(word: 4) * BigUInt(low: 1, high: 2), .inline(4, 8), [4, 8])

            XCTAssertEqual(
                BigUInt(words: [1, 2, 3, 4]) * BigUInt(),
                BigUInt())
            XCTAssertEqual(
                BigUInt() * BigUInt(words: [1, 2, 3, 4]),
                BigUInt())
            XCTAssertEqual(
                BigUInt(words: [1, 2, 3, 4]) * BigUInt(words: [2]),
                BigUInt(words: [2, 4, 6, 8]))
            XCTAssertEqual(
                BigUInt(words: [1, 2, 3, 4]).multiplied(by: BigUInt(words: [2])),
                BigUInt(words: [2, 4, 6, 8]))
            XCTAssertEqual(
                BigUInt(words: [2]) * BigUInt(words: [1, 2, 3, 4]),
                BigUInt(words: [2, 4, 6, 8]))
            XCTAssertEqual(
                BigUInt(words: [1, 2, 3, 4]) * BigUInt(words: [0, 1]),
                BigUInt(words: [0, 1, 2, 3, 4]))
            XCTAssertEqual(
                BigUInt(words: [0, 1]) * BigUInt(words: [1, 2, 3, 4]),
                BigUInt(words: [0, 1, 2, 3, 4]))
            XCTAssertEqual(
                BigUInt(words: [4, 3, 2, 1]) * BigUInt(words: [1, 2, 3, 4]),
                BigUInt(words: [4, 11, 20, 30, 20, 11, 4]))
            // 999 * 99 = 98901
            XCTAssertEqual(
                BigUInt(words: [Word.max, Word.max, Word.max]) * BigUInt(words: [Word.max, Word.max]),
                BigUInt(words: [1, 0, Word.max, Word.max - 1, Word.max]))
            XCTAssertEqual(
                BigUInt(words: [1, 2]) * BigUInt(words: [2, 1]),
                BigUInt(words: [2, 5, 2]))

            var b = BigUInt("2637AB28", radix: 16)!
            b *= BigUInt("164B", radix: 16)!
            XCTAssertEqual(b, BigUInt("353FB0494B8", radix: 16))

            XCTAssertEqual(BigUInt("16B60", radix: 16)! * BigUInt("33E28", radix: 16)!, BigUInt("49A5A0700", radix: 16)!)
        }

        test()
        // Disable brute force multiplication.
//        let limit = BigUInt.directMultiplicationLimit
//        BigUInt.directMultiplicationLimit = 0
//        defer { BigUInt.directMultiplicationLimit = limit }
//
//        test()
    }

    func testDivision() {
        func test(_ a: [Word], _ b: [Word], file: StaticString = #filePath, line: UInt = #line) {
            let x = BigUInt(words: a)
            let y = BigUInt(words: b)
            let (div, mod) = x.quotientAndRemainder(dividingBy: y)
            if mod >= y {
                XCTFail("x:\(x) = div:\(div) * y:\(y) + mod:\(mod)", file: file, line: line)
            }
            if div * y + mod != x {
                XCTFail("x:\(x) = div:\(div) * y:\(y) + mod:\(mod)", file: file, line: line)
            }

            let shift = y.leadingZeroBitCount
            let norm = y << shift
            var rem = x
            rem.formRemainder(dividingBy: norm, normalizedBy: shift)
            XCTAssertEqual(rem, mod, file: file, line: line)
        }

        // These cases exercise all code paths in the division when Word is UInt8 or UInt64.
        test([], [1])
        test([1], [1])
        test([1], [2])
        test([2], [1])
        test([], [0, 1])
        test([1], [0, 1])
        test([0, 1], [0, 1])
        test([0, 0, 1], [0, 1])
        test([0, 0, 1], [1, 1])
        test([0, 0, 1], [3, 1])
        test([0, 0, 1], [75, 1])
        test([0, 0, 0, 1], [0, 1])
        test([2, 4, 6, 8], [1, 2])
        test([2, 3, 4, 5], [4, 5])
        test([Word.max, Word.max - 1, Word.max], [Word.max, Word.max])
        test([0, Word.max, Word.max - 1], [Word.max, Word.max])
        test([0, 0, 0, 0, 0, Word.max / 2 + 1, Word.max / 2], [1, 0, 0, Word.max / 2 + 1])
        test([0, Word.max - 1, Word.max / 2 + 1], [Word.max, Word.max / 2 + 1])
        test([0, 0, 0x41 << Word(Word.bitWidth - 8)], [Word.max, 1 << Word(Word.bitWidth - 1)])

        XCTAssertEqual(BigUInt(328) / BigUInt(21), BigUInt(15))
        XCTAssertEqual(BigUInt(328) % BigUInt(21), BigUInt(13))

        var a = BigUInt(328)
        a /= 21
        XCTAssertEqual(a, 15)
        a %= 7
        XCTAssertEqual(a, 1)

        #if false
            for x0 in (0 ... Int(Word.max)) {
                for x1 in (0 ... Int(Word.max)).reverse() {
                    for y0 in (0 ... Int(Word.max)).reverse() {
                        for y1 in (1 ... Int(Word.max)).reverse() {
                            for x2 in (1 ... y1).reverse() {
                                test(
                                    [Word(x0), Word(x1), Word(x2)],
                                    [Word(y0), Word(y1)])
                            }
                        }
                    }
                }
            }
        #endif
    }

    func testFactorial() {
        let power = 10
        var forward = BigUInt(1)
        for i in 1 ..< (1 << power) {
            forward *= BigUInt(i)
        }
        print("\(1 << power - 1)! = \(forward) [\(forward.count)]")
        var backward = BigUInt(1)
        for i in (1 ..< (1 << power)).reversed() {
            backward *= BigUInt(i)
        }

        func balancedFactorial(level: Int, offset: Int) -> BigUInt {
            if level == 0 {
                return BigUInt(offset == 0 ? 1 : offset)
            }
            let a = balancedFactorial(level: level - 1, offset: 2 * offset)
            let b = balancedFactorial(level: level - 1, offset: 2 * offset + 1)
            return a * b
        }
        let balanced = balancedFactorial(level: power, offset: 0)

        XCTAssertEqual(backward, forward)
        XCTAssertEqual(balanced, forward)

        var remaining = balanced
        for i in 1 ..< (1 << power) {
            let (div, mod) = remaining.quotientAndRemainder(dividingBy: BigUInt(i))
            XCTAssertEqual(mod, 0)
            remaining = div
        }
        XCTAssertEqual(remaining, 1)
    }

    func testExponentiation() {
        XCTAssertEqual(BigUInt(0).power(0), BigUInt(1))
        XCTAssertEqual(BigUInt(0).power(1), BigUInt(0))

        XCTAssertEqual(BigUInt(1).power(0), BigUInt(1))
        XCTAssertEqual(BigUInt(1).power(1), BigUInt(1))
        XCTAssertEqual(BigUInt(1).power(-1), BigUInt(1))
        XCTAssertEqual(BigUInt(1).power(-2), BigUInt(1))
        XCTAssertEqual(BigUInt(1).power(-3), BigUInt(1))
        XCTAssertEqual(BigUInt(1).power(-4), BigUInt(1))

        XCTAssertEqual(BigUInt(2).power(0), BigUInt(1))
        XCTAssertEqual(BigUInt(2).power(1), BigUInt(2))
        XCTAssertEqual(BigUInt(2).power(2), BigUInt(4))
        XCTAssertEqual(BigUInt(2).power(3), BigUInt(8))
        XCTAssertEqual(BigUInt(2).power(-1), BigUInt(0))
        XCTAssertEqual(BigUInt(2).power(-2), BigUInt(0))
        XCTAssertEqual(BigUInt(2).power(-3), BigUInt(0))

        XCTAssertEqual(BigUInt(3).power(0), BigUInt(1))
        XCTAssertEqual(BigUInt(3).power(1), BigUInt(3))
        XCTAssertEqual(BigUInt(3).power(2), BigUInt(9))
        XCTAssertEqual(BigUInt(3).power(3), BigUInt(27))
        XCTAssertEqual(BigUInt(3).power(-1), BigUInt(0))
        XCTAssertEqual(BigUInt(3).power(-2), BigUInt(0))

        XCTAssertEqual((BigUInt(1) << 256).power(0), BigUInt(1))
        XCTAssertEqual((BigUInt(1) << 256).power(1), BigUInt(1) << 256)
        XCTAssertEqual((BigUInt(1) << 256).power(2), BigUInt(1) << 512)

        XCTAssertEqual(BigUInt(0).power(577), BigUInt(0))
        XCTAssertEqual(BigUInt(1).power(577), BigUInt(1))
        XCTAssertEqual(BigUInt(2).power(577), BigUInt(1) << 577)
    }

    func testBitWidth() {
        XCTAssertEqual(BigUInt(0).bitWidth, 0)
        XCTAssertEqual(BigUInt(1).bitWidth, 1)
        XCTAssertEqual(BigUInt(Word.max).bitWidth, Word.bitWidth)
        XCTAssertEqual(BigUInt(words: [Word.max, 1]).bitWidth, Word.bitWidth + 1)
        XCTAssertEqual(BigUInt(words: [2, 12]).bitWidth, Word.bitWidth + 4)
        XCTAssertEqual(BigUInt(words: [1, Word.max]).bitWidth, 2 * Word.bitWidth)

        XCTAssertEqual(BigUInt(0).leadingZeroBitCount, 0)
        XCTAssertEqual(BigUInt(1).leadingZeroBitCount, Word.bitWidth - 1)
        XCTAssertEqual(BigUInt(Word.max).leadingZeroBitCount, 0)
        XCTAssertEqual(BigUInt(words: [Word.max, 1]).leadingZeroBitCount, Word.bitWidth - 1)
        XCTAssertEqual(BigUInt(words: [14, Word.max]).leadingZeroBitCount, 0)

        XCTAssertEqual(BigUInt(0).trailingZeroBitCount, 0)
        XCTAssertEqual(BigUInt((1 as Word) << (Word.bitWidth - 1)).trailingZeroBitCount, Word.bitWidth - 1)
        XCTAssertEqual(BigUInt(Word.max).trailingZeroBitCount, 0)
        XCTAssertEqual(BigUInt(words: [0, 1]).trailingZeroBitCount, Word.bitWidth)
        XCTAssertEqual(BigUInt(words: [0, 1 << Word(Word.bitWidth - 1)]).trailingZeroBitCount, 2 * Word.bitWidth - 1)
    }

    func testBitwise() {
        let a = BigUInt("1234567890ABCDEF13579BDF2468ACE", radix: 16)!
        let b = BigUInt("ECA8642FDB97531FEDCBA0987654321", radix: 16)!

        //                                    a = 01234567890ABCDEF13579BDF2468ACE
        //                                    b = 0ECA8642FDB97531FEDCBA0987654321
        XCTAssertEqual(String(~a,    radix: 16), "fedcba9876f543210eca86420db97531")
        XCTAssertEqual(String(a | b, radix: 16),  "febc767fdbbfdfffffdfbbdf767cbef")
        XCTAssertEqual(String(a & b, radix: 16),    "2044289083410f014380982440200")
        XCTAssertEqual(String(a ^ b, radix: 16),  "fe9c32574b3c9ef0fe9c3b47523c9ef")

        let ffff = BigUInt(words: Array(repeating: Word.max, count: 30))
        let not = ~ffff
        let zero = BigUInt()
        XCTAssertEqual(not, zero)
        XCTAssertEqual(Array((~ffff).words), [])
        XCTAssertEqual(a | ffff, ffff)
        XCTAssertEqual(a | 0, a)
        XCTAssertEqual(a & a, a)
        XCTAssertEqual(a & 0, 0)
        XCTAssertEqual(a & ffff, a)
        XCTAssertEqual(~(a | b), (~a & ~b))
        XCTAssertEqual(~(a & b), (~a | ~b).extract(..<(a&b).count))
        XCTAssertEqual(a ^ a, 0)
        XCTAssertEqual((a ^ b) ^ b, a)
        XCTAssertEqual((a ^ b) ^ a, b)

        var z = a * b
        z |= a
        z &= b
        z ^= ffff
        XCTAssertEqual(z, (((a * b) | a) & b) ^ ffff)
    }

    func testLeftShifts() {
        let sample = BigUInt("123456789ABCDEF01234567891631832727633", radix: 16)!

        var a = sample

        a <<= 0
        XCTAssertEqual(a, sample)

        a = sample
        a <<= 1
        XCTAssertEqual(a, 2 * sample)

        a = sample
        a <<= Word.bitWidth
        XCTAssertEqual(a.count, sample.count + 1)
        XCTAssertEqual(a[0], 0)
        XCTAssertEqual(a.extract(1 ... sample.count + 1), sample)

        a = sample
        a <<= 100 * Word.bitWidth
        XCTAssertEqual(a.count, sample.count + 100)
        XCTAssertEqual(a.extract(0 ..< 100), 0)
        XCTAssertEqual(a.extract(100 ... sample.count + 100), sample)

        a = sample
        a <<= 100 * Word.bitWidth + 2
        XCTAssertEqual(a.count, sample.count + 100)
        XCTAssertEqual(a.extract(0 ..< 100), 0)
        XCTAssertEqual(a.extract(100 ... sample.count + 100), sample << 2)

        a = sample
        a <<= Word.bitWidth - 1
        XCTAssertEqual(a.count, sample.count + 1)
        XCTAssertEqual(a, BigUInt(words: [0] + sample.words) / 2)


        a = sample
        a <<= -4
        XCTAssertEqual(a, sample / 16)

        XCTAssertEqual(sample << 0, sample)
        XCTAssertEqual(sample << 1, 2 * sample)
        XCTAssertEqual(sample << 2, 4 * sample)
        XCTAssertEqual(sample << 4, 16 * sample)
        XCTAssertEqual(sample << Word.bitWidth, BigUInt(words: [0 as Word] + sample.words))
        XCTAssertEqual(sample << (Word.bitWidth - 1), BigUInt(words: [0] + sample.words) / 2)
        XCTAssertEqual(sample << (Word.bitWidth + 1), BigUInt(words: [0] + sample.words) * 2)
        XCTAssertEqual(sample << (Word.bitWidth + 2), BigUInt(words: [0] + sample.words) * 4)
        XCTAssertEqual(sample << (2 * Word.bitWidth), BigUInt(words: [0, 0] + sample.words))
        XCTAssertEqual(sample << (2 * Word.bitWidth + 2), BigUInt(words: [0, 0] + (4 * sample).words))

        XCTAssertEqual(sample << -1, sample / 2)
        XCTAssertEqual(sample << -4, sample / 16)
    }

    func testRightShifts() {
        let sample = BigUInt("123456789ABCDEF1234567891631832727633", radix: 16)!

        var a = sample

        a >>= BigUInt(0)
        XCTAssertEqual(a, sample)

        a >>= 0
        XCTAssertEqual(a, sample)

        a = sample
        a >>= 1
        XCTAssertEqual(a, sample / 2)

        a = sample
        a >>= Word.bitWidth
        XCTAssertEqual(a, sample.extract(1...))

        a = sample
        a >>= Word.bitWidth + 2
        XCTAssertEqual(a, sample.extract(1...) / 4)

        a = sample
        a >>= sample.count * Word.bitWidth
        XCTAssertEqual(a, 0)

        a = sample
        a >>= 1000
        XCTAssertEqual(a, 0)

        a = sample
        a >>= 100 * Word.bitWidth
        XCTAssertEqual(a, 0)

        a = sample
        a >>= 100 * BigUInt(Word.max)
        XCTAssertEqual(a, 0)

        a = sample
        a >>= -1
        XCTAssertEqual(a, sample * 2)

        a = sample
        a >>= -4
        XCTAssertEqual(a, sample * 16)

        XCTAssertEqual(sample >> BigUInt(0), sample)
        XCTAssertEqual(sample >> 0, sample)
        XCTAssertEqual(sample >> 1, sample / 2)
        XCTAssertEqual(sample >> 3, sample / 8)
        XCTAssertEqual(sample >> Word.bitWidth, sample.extract(1 ..< sample.count))
        XCTAssertEqual(sample >> (Word.bitWidth + 2), sample.extract(1...) / 4)
        XCTAssertEqual(sample >> (Word.bitWidth + 3), sample.extract(1...) / 8)
        XCTAssertEqual(sample >> (sample.count * Word.bitWidth), 0)
        XCTAssertEqual(sample >> (100 * Word.bitWidth), 0)
        XCTAssertEqual(sample >> (100 * BigUInt(Word.max)), 0)

        XCTAssertEqual(sample >> -1, sample * 2)
        XCTAssertEqual(sample >> -4, sample * 16)
    }

    func testConversionToString() {
        let sample = BigUInt("123456789ABCDEFEDCBA98765432123456789ABCDEF", radix: 16)!
        // Radix = 10
        XCTAssertEqual(String(BigUInt()), "0")
        XCTAssertEqual(String(BigUInt(1)), "1")
        XCTAssertEqual(String(BigUInt(100)), "100")
        XCTAssertEqual(String(BigUInt(12345)), "12345")
        XCTAssertEqual(String(BigUInt(123456789)), "123456789")
        XCTAssertEqual(String(sample), "425693205796080237694414176550132631862392541400559")

        // Radix = 16
        XCTAssertEqual(String(BigUInt(0x1001), radix: 16), "1001")
        XCTAssertEqual(String(BigUInt(0x0102030405060708), radix: 16), "102030405060708")
        XCTAssertEqual(String(sample, radix: 16), "123456789abcdefedcba98765432123456789abcdef")
        XCTAssertEqual(String(sample, radix: 16, uppercase: true), "123456789ABCDEFEDCBA98765432123456789ABCDEF")

        // Radix = 2
        XCTAssertEqual(String(BigUInt(12), radix: 2), "1100")
        XCTAssertEqual(String(BigUInt(123), radix: 2), "1111011")
        XCTAssertEqual(String(BigUInt(1234), radix: 2), "10011010010")
        XCTAssertEqual(String(sample, radix: 2), "1001000110100010101100111100010011010101111001101111011111110110111001011101010011000011101100101010000110010000100100011010001010110011110001001101010111100110111101111")

        // Radix = 31
        XCTAssertEqual(String(BigUInt(30), radix: 31), "u")
        XCTAssertEqual(String(BigUInt(31), radix: 31), "10")
        XCTAssertEqual(String(BigUInt("10000000000000000", radix: 16)!, radix: 31), "nd075ib45k86g")
        XCTAssertEqual(String(BigUInt("2908B5129F59DB6A41", radix: 16)!, radix: 31), "100000000000000")
        XCTAssertEqual(String(sample, radix: 31), "ptf96helfaqi7ogc3jbonmccrhmnc2b61s")

        let quickLook = BigUInt(513).playgroundDescription as? String
        if quickLook == "513 (10 bits)" {
        } else {
            XCTFail("Unexpected playground QuickLook representation: \(quickLook ?? "nil")")
        }
    }

    func testConversionFromString() {
        let sample = "123456789ABCDEFEDCBA98765432123456789ABCDEF"

        XCTAssertEqual(BigUInt("1"), 1)
        XCTAssertEqual(BigUInt("123456789ABCDEF", radix: 16)!, 0x123456789ABCDEF)
        XCTAssertEqual(BigUInt("1000000000000000000000"), BigUInt("3635C9ADC5DEA00000", radix: 16))
        XCTAssertEqual(BigUInt("10000000000000000", radix: 16), BigUInt("18446744073709551616"))
        XCTAssertEqual(BigUInt(sample, radix: 16)!, BigUInt("425693205796080237694414176550132631862392541400559"))

        // We have to call BigUInt.init here because we don't want Literal initialization via coercion (SE-0213)
        XCTAssertNil(BigUInt.init("Not a number"))
        XCTAssertNil(BigUInt.init("X"))
        XCTAssertNil(BigUInt.init("12349A"))
        XCTAssertNil(BigUInt.init("000000000000000000000000A000"))
        XCTAssertNil(BigUInt.init("00A0000000000000000000000000"))
        XCTAssertNil(BigUInt.init("00 0000000000000000000000000"))
        XCTAssertNil(BigUInt.init("\u{4e00}\u{4e03}")) // Chinese numerals "1", "7"

        XCTAssertEqual(BigUInt("u", radix: 31)!, 30)
        XCTAssertEqual(BigUInt("10", radix: 31)!, 31)
        XCTAssertEqual(BigUInt("100000000000000", radix: 31)!, BigUInt("2908B5129F59DB6A41", radix: 16)!)
        XCTAssertEqual(BigUInt("nd075ib45k86g", radix: 31)!, BigUInt("10000000000000000", radix: 16)!)
        XCTAssertEqual(BigUInt("ptf96helfaqi7ogc3jbonmccrhmnc2b61s", radix: 31)!, BigUInt(sample, radix: 16)!)

        XCTAssertNotNil(BigUInt(sample.repeated(100), radix: 16))
   }
}

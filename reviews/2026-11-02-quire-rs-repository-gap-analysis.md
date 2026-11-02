---
id: SR-16291
title: "Gap analysis — quire-rs repository audit"
type: SpecReview
analysis: gap-analysis
scope: "spec/, src/, tests/, tools/"
review_set: all
relationships:
  - target: "ix://agent-ix/quire-rs/FR-069"
    type: references
---

## Summary

Repository-driven audit of the full quire-rs specification, source, and test tree. Plan completion was not assessed. The computed matrix reports 293 untagged acceptance criteria and no run evidence for 1015 criteria.

## Verdict

**FAIL** — untagged criteria and no-run-evidence findings remain in this repository-wide, planless audit.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-001-AC-3 has no test or symbol bound by the computed matrix | FR-001-AC-3 |
| FND-002 | high | FR-001-AC-4 has no test or symbol bound by the computed matrix | FR-001-AC-4 |
| FND-003 | high | FR-002-AC-5 has no test or symbol bound by the computed matrix | FR-002-AC-5 |
| FND-004 | high | FR-002-AC-7 has no test or symbol bound by the computed matrix | FR-002-AC-7 |
| FND-005 | high | FR-003-AC-3 has no test or symbol bound by the computed matrix | FR-003-AC-3 |
| FND-006 | high | FR-004-AC-1 has no test or symbol bound by the computed matrix | FR-004-AC-1 |
| FND-007 | high | FR-004-AC-2 has no test or symbol bound by the computed matrix | FR-004-AC-2 |
| FND-008 | high | FR-004-AC-3 has no test or symbol bound by the computed matrix | FR-004-AC-3 |
| FND-009 | high | FR-004-AC-4 has no test or symbol bound by the computed matrix | FR-004-AC-4 |
| FND-010 | high | FR-009-AC-5 has no test or symbol bound by the computed matrix | FR-009-AC-5 |
| FND-011 | high | FR-010-AC-1 has no test or symbol bound by the computed matrix | FR-010-AC-1 |
| FND-012 | high | FR-010-AC-2 has no test or symbol bound by the computed matrix | FR-010-AC-2 |
| FND-013 | high | FR-011-AC-1 has no test or symbol bound by the computed matrix | FR-011-AC-1 |
| FND-014 | high | FR-011-AC-4 has no test or symbol bound by the computed matrix | FR-011-AC-4 |
| FND-015 | high | FR-011-AC-5 has no test or symbol bound by the computed matrix | FR-011-AC-5 |
| FND-016 | high | FR-012-AC-1 has no test or symbol bound by the computed matrix | FR-012-AC-1 |
| FND-017 | high | FR-012-AC-2 has no test or symbol bound by the computed matrix | FR-012-AC-2 |
| FND-018 | high | FR-012-AC-3 has no test or symbol bound by the computed matrix | FR-012-AC-3 |
| FND-019 | high | FR-012-AC-4 has no test or symbol bound by the computed matrix | FR-012-AC-4 |
| FND-020 | high | FR-012-AC-5 has no test or symbol bound by the computed matrix | FR-012-AC-5 |
| FND-021 | high | FR-013-AC-1 has no test or symbol bound by the computed matrix | FR-013-AC-1 |
| FND-022 | high | FR-013-AC-2 has no test or symbol bound by the computed matrix | FR-013-AC-2 |
| FND-023 | high | FR-013-AC-4 has no test or symbol bound by the computed matrix | FR-013-AC-4 |
| FND-024 | high | FR-013-AC-8 has no test or symbol bound by the computed matrix | FR-013-AC-8 |
| FND-025 | high | FR-013-AC-9 has no test or symbol bound by the computed matrix | FR-013-AC-9 |
| FND-026 | high | FR-013-AC-10 has no test or symbol bound by the computed matrix | FR-013-AC-10 |
| FND-027 | high | FR-013-AC-14 has no test or symbol bound by the computed matrix | FR-013-AC-14 |
| FND-028 | high | FR-013-CFG-1 has no test or symbol bound by the computed matrix | FR-013-CFG-1 |
| FND-029 | high | FR-014-AC-1 has no test or symbol bound by the computed matrix | FR-014-AC-1 |
| FND-030 | high | FR-014-AC-5 has no test or symbol bound by the computed matrix | FR-014-AC-5 |
| FND-031 | high | FR-014-AC-6 has no test or symbol bound by the computed matrix | FR-014-AC-6 |
| FND-032 | high | FR-016-AC-1 has no test or symbol bound by the computed matrix | FR-016-AC-1 |
| FND-033 | high | FR-016-AC-2 has no test or symbol bound by the computed matrix | FR-016-AC-2 |
| FND-034 | high | FR-016-AC-3 has no test or symbol bound by the computed matrix | FR-016-AC-3 |
| FND-035 | high | FR-016-AC-4 has no test or symbol bound by the computed matrix | FR-016-AC-4 |
| FND-036 | high | FR-019-CON-1 has no test or symbol bound by the computed matrix | FR-019-CON-1 |
| FND-037 | high | FR-020-CON-1 has no test or symbol bound by the computed matrix | FR-020-CON-1 |
| FND-038 | high | FR-022-AC-6 has no test or symbol bound by the computed matrix | FR-022-AC-6 |
| FND-039 | high | FR-022-AC-7 has no test or symbol bound by the computed matrix | FR-022-AC-7 |
| FND-040 | high | FR-023-AC-2 has no test or symbol bound by the computed matrix | FR-023-AC-2 |
| FND-041 | high | FR-023-AC-3 has no test or symbol bound by the computed matrix | FR-023-AC-3 |
| FND-042 | high | FR-023-AC-4 has no test or symbol bound by the computed matrix | FR-023-AC-4 |
| FND-043 | high | FR-023-AC-5 has no test or symbol bound by the computed matrix | FR-023-AC-5 |
| FND-044 | high | FR-023-AC-6 has no test or symbol bound by the computed matrix | FR-023-AC-6 |
| FND-045 | high | FR-024-AC-1 has no test or symbol bound by the computed matrix | FR-024-AC-1 |
| FND-046 | high | FR-024-AC-5 has no test or symbol bound by the computed matrix | FR-024-AC-5 |
| FND-047 | high | FR-024-AC-8 has no test or symbol bound by the computed matrix | FR-024-AC-8 |
| FND-048 | high | FR-026-AC-7 has no test or symbol bound by the computed matrix | FR-026-AC-7 |
| FND-049 | high | FR-026-CON-1 has no test or symbol bound by the computed matrix | FR-026-CON-1 |
| FND-050 | high | FR-027-AC-1 has no test or symbol bound by the computed matrix | FR-027-AC-1 |
| FND-051 | high | FR-027-AC-2 has no test or symbol bound by the computed matrix | FR-027-AC-2 |
| FND-052 | high | FR-027-AC-3 has no test or symbol bound by the computed matrix | FR-027-AC-3 |
| FND-053 | high | FR-027-AC-4 has no test or symbol bound by the computed matrix | FR-027-AC-4 |
| FND-054 | high | FR-027-AC-7 has no test or symbol bound by the computed matrix | FR-027-AC-7 |
| FND-055 | high | FR-027-AC-8 has no test or symbol bound by the computed matrix | FR-027-AC-8 |
| FND-056 | high | FR-028-AC-2 has no test or symbol bound by the computed matrix | FR-028-AC-2 |
| FND-057 | high | FR-028-AC-3 has no test or symbol bound by the computed matrix | FR-028-AC-3 |
| FND-058 | high | FR-028-AC-4 has no test or symbol bound by the computed matrix | FR-028-AC-4 |
| FND-059 | high | FR-028-AC-5 has no test or symbol bound by the computed matrix | FR-028-AC-5 |
| FND-060 | high | FR-028-AC-6 has no test or symbol bound by the computed matrix | FR-028-AC-6 |
| FND-061 | high | FR-028-AC-7 has no test or symbol bound by the computed matrix | FR-028-AC-7 |
| FND-062 | high | FR-028-AC-8 has no test or symbol bound by the computed matrix | FR-028-AC-8 |
| FND-063 | high | FR-028-AC-9 has no test or symbol bound by the computed matrix | FR-028-AC-9 |
| FND-064 | high | FR-028-AC-10 has no test or symbol bound by the computed matrix | FR-028-AC-10 |
| FND-065 | high | FR-030-AC-1 has no test or symbol bound by the computed matrix | FR-030-AC-1 |
| FND-066 | high | FR-030-AC-2 has no test or symbol bound by the computed matrix | FR-030-AC-2 |
| FND-067 | high | FR-030-AC-3 has no test or symbol bound by the computed matrix | FR-030-AC-3 |
| FND-068 | high | FR-030-AC-4 has no test or symbol bound by the computed matrix | FR-030-AC-4 |
| FND-069 | high | FR-030-AC-5 has no test or symbol bound by the computed matrix | FR-030-AC-5 |
| FND-070 | high | FR-030-AC-6 has no test or symbol bound by the computed matrix | FR-030-AC-6 |
| FND-071 | high | FR-033-AC-14 has no test or symbol bound by the computed matrix | FR-033-AC-14 |
| FND-072 | high | FR-033-AC-15 has no test or symbol bound by the computed matrix | FR-033-AC-15 |
| FND-073 | high | FR-036-AC-6 has no test or symbol bound by the computed matrix | FR-036-AC-6 |
| FND-074 | high | FR-037-AC-6 has no test or symbol bound by the computed matrix | FR-037-AC-6 |
| FND-075 | high | FR-038-AC-8 has no test or symbol bound by the computed matrix | FR-038-AC-8 |
| FND-076 | high | FR-042-AC-10 has no test or symbol bound by the computed matrix | FR-042-AC-10 |
| FND-077 | high | FR-043-AC-7 has no test or symbol bound by the computed matrix | FR-043-AC-7 |
| FND-078 | high | FR-043-CON-1 has no test or symbol bound by the computed matrix | FR-043-CON-1 |
| FND-079 | high | FR-043-CON-2 has no test or symbol bound by the computed matrix | FR-043-CON-2 |
| FND-080 | high | FR-044-CON-1 has no test or symbol bound by the computed matrix | FR-044-CON-1 |
| FND-081 | high | FR-044-CON-2 has no test or symbol bound by the computed matrix | FR-044-CON-2 |
| FND-082 | high | FR-045-CON-2 has no test or symbol bound by the computed matrix | FR-045-CON-2 |
| FND-083 | high | FR-045-CON-3 has no test or symbol bound by the computed matrix | FR-045-CON-3 |
| FND-084 | high | FR-045-CON-4 has no test or symbol bound by the computed matrix | FR-045-CON-4 |
| FND-085 | high | FR-046-AC-1 has no test or symbol bound by the computed matrix | FR-046-AC-1 |
| FND-086 | high | FR-046-CON-2 has no test or symbol bound by the computed matrix | FR-046-CON-2 |
| FND-087 | high | FR-047-AC-9 has no test or symbol bound by the computed matrix | FR-047-AC-9 |
| FND-088 | high | FR-050-AC-14 has no test or symbol bound by the computed matrix | FR-050-AC-14 |
| FND-089 | high | FR-050-AC-20 has no test or symbol bound by the computed matrix | FR-050-AC-20 |
| FND-090 | high | FR-051-AC-23 has no test or symbol bound by the computed matrix | FR-051-AC-23 |
| FND-091 | high | FR-051-AC-24 has no test or symbol bound by the computed matrix | FR-051-AC-24 |
| FND-092 | high | FR-051-AC-26 has no test or symbol bound by the computed matrix | FR-051-AC-26 |
| FND-093 | high | FR-051-CON-2 has no test or symbol bound by the computed matrix | FR-051-CON-2 |
| FND-094 | high | FR-052-AC-11 has no test or symbol bound by the computed matrix | FR-052-AC-11 |
| FND-095 | high | FR-052-CON-1 has no test or symbol bound by the computed matrix | FR-052-CON-1 |
| FND-096 | high | FR-052-CON-4 has no test or symbol bound by the computed matrix | FR-052-CON-4 |
| FND-097 | high | FR-053-AC-4 has no test or symbol bound by the computed matrix | FR-053-AC-4 |
| FND-098 | high | FR-053-AC-12 has no test or symbol bound by the computed matrix | FR-053-AC-12 |
| FND-099 | high | FR-053-AC-15 has no test or symbol bound by the computed matrix | FR-053-AC-15 |
| FND-100 | high | FR-053-AC-16 has no test or symbol bound by the computed matrix | FR-053-AC-16 |
| FND-101 | high | FR-053-AC-17 has no test or symbol bound by the computed matrix | FR-053-AC-17 |
| FND-102 | high | FR-053-AC-18 has no test or symbol bound by the computed matrix | FR-053-AC-18 |
| FND-103 | high | FR-053-AC-19 has no test or symbol bound by the computed matrix | FR-053-AC-19 |
| FND-104 | high | FR-053-AC-20 has no test or symbol bound by the computed matrix | FR-053-AC-20 |
| FND-105 | high | FR-053-AC-21 has no test or symbol bound by the computed matrix | FR-053-AC-21 |
| FND-106 | high | FR-053-AC-22 has no test or symbol bound by the computed matrix | FR-053-AC-22 |
| FND-107 | high | FR-053-AC-23 has no test or symbol bound by the computed matrix | FR-053-AC-23 |
| FND-108 | high | FR-053-CON-3 has no test or symbol bound by the computed matrix | FR-053-CON-3 |
| FND-109 | high | FR-053-CON-4 has no test or symbol bound by the computed matrix | FR-053-CON-4 |
| FND-110 | high | FR-054-CON-3 has no test or symbol bound by the computed matrix | FR-054-CON-3 |
| FND-111 | high | FR-054-CON-4 has no test or symbol bound by the computed matrix | FR-054-CON-4 |
| FND-112 | high | FR-054-CON-5 has no test or symbol bound by the computed matrix | FR-054-CON-5 |
| FND-113 | high | FR-055-CON-1 has no test or symbol bound by the computed matrix | FR-055-CON-1 |
| FND-114 | high | FR-055-CON-2 has no test or symbol bound by the computed matrix | FR-055-CON-2 |
| FND-115 | high | FR-056-CON-1 has no test or symbol bound by the computed matrix | FR-056-CON-1 |
| FND-116 | high | FR-056-CON-2 has no test or symbol bound by the computed matrix | FR-056-CON-2 |
| FND-117 | high | FR-056-CON-4 has no test or symbol bound by the computed matrix | FR-056-CON-4 |
| FND-118 | high | FR-056-CON-5 has no test or symbol bound by the computed matrix | FR-056-CON-5 |
| FND-119 | high | FR-057-AC-1 has no test or symbol bound by the computed matrix | FR-057-AC-1 |
| FND-120 | high | FR-057-AC-2 has no test or symbol bound by the computed matrix | FR-057-AC-2 |
| FND-121 | high | FR-057-AC-3 has no test or symbol bound by the computed matrix | FR-057-AC-3 |
| FND-122 | high | FR-057-AC-7 has no test or symbol bound by the computed matrix | FR-057-AC-7 |
| FND-123 | high | FR-057-AC-8 has no test or symbol bound by the computed matrix | FR-057-AC-8 |
| FND-124 | high | FR-057-AC-9 has no test or symbol bound by the computed matrix | FR-057-AC-9 |
| FND-125 | high | FR-058-AC-6 has no test or symbol bound by the computed matrix | FR-058-AC-6 |
| FND-126 | high | FR-058-AC-7 has no test or symbol bound by the computed matrix | FR-058-AC-7 |
| FND-127 | high | FR-058-AC-9 has no test or symbol bound by the computed matrix | FR-058-AC-9 |
| FND-128 | high | FR-062-AC-12 has no test or symbol bound by the computed matrix | FR-062-AC-12 |
| FND-129 | high | FR-062-AC-13 has no test or symbol bound by the computed matrix | FR-062-AC-13 |
| FND-130 | high | FR-062-AC-14 has no test or symbol bound by the computed matrix | FR-062-AC-14 |
| FND-131 | high | FR-062-AC-15 has no test or symbol bound by the computed matrix | FR-062-AC-15 |
| FND-132 | high | FR-062-AC-16 has no test or symbol bound by the computed matrix | FR-062-AC-16 |
| FND-133 | high | FR-062-AC-17 has no test or symbol bound by the computed matrix | FR-062-AC-17 |
| FND-134 | high | FR-062-CON-1 has no test or symbol bound by the computed matrix | FR-062-CON-1 |
| FND-135 | high | FR-064-CON-2 has no test or symbol bound by the computed matrix | FR-064-CON-2 |
| FND-136 | high | FR-064-CON-3 has no test or symbol bound by the computed matrix | FR-064-CON-3 |
| FND-137 | high | FR-064-CON-4 has no test or symbol bound by the computed matrix | FR-064-CON-4 |
| FND-138 | high | FR-064-CON-5 has no test or symbol bound by the computed matrix | FR-064-CON-5 |
| FND-139 | high | FR-064-CON-6 has no test or symbol bound by the computed matrix | FR-064-CON-6 |
| FND-140 | high | FR-065-AC-2 has no test or symbol bound by the computed matrix | FR-065-AC-2 |
| FND-141 | high | FR-065-AC-4 has no test or symbol bound by the computed matrix | FR-065-AC-4 |
| FND-142 | high | FR-065-AC-7 has no test or symbol bound by the computed matrix | FR-065-AC-7 |
| FND-143 | high | FR-065-AC-8 has no test or symbol bound by the computed matrix | FR-065-AC-8 |
| FND-144 | high | FR-065-AC-9 has no test or symbol bound by the computed matrix | FR-065-AC-9 |
| FND-145 | high | FR-065-AC-10 has no test or symbol bound by the computed matrix | FR-065-AC-10 |
| FND-146 | high | FR-065-AC-14 has no test or symbol bound by the computed matrix | FR-065-AC-14 |
| FND-147 | high | FR-065-AC-16 has no test or symbol bound by the computed matrix | FR-065-AC-16 |
| FND-148 | high | FR-065-AC-24 has no test or symbol bound by the computed matrix | FR-065-AC-24 |
| FND-149 | high | FR-065-CON-1 has no test or symbol bound by the computed matrix | FR-065-CON-1 |
| FND-150 | high | FR-065-CON-2 has no test or symbol bound by the computed matrix | FR-065-CON-2 |
| FND-151 | high | FR-066-AC-1 has no test or symbol bound by the computed matrix | FR-066-AC-1 |
| FND-152 | high | FR-066-AC-4 has no test or symbol bound by the computed matrix | FR-066-AC-4 |
| FND-153 | high | FR-066-AC-5 has no test or symbol bound by the computed matrix | FR-066-AC-5 |
| FND-154 | high | FR-066-AC-7 has no test or symbol bound by the computed matrix | FR-066-AC-7 |
| FND-155 | high | FR-066-AC-8 has no test or symbol bound by the computed matrix | FR-066-AC-8 |
| FND-156 | high | FR-066-AC-9 has no test or symbol bound by the computed matrix | FR-066-AC-9 |
| FND-157 | high | FR-066-AC-10 has no test or symbol bound by the computed matrix | FR-066-AC-10 |
| FND-158 | high | FR-067-AC-6 has no test or symbol bound by the computed matrix | FR-067-AC-6 |
| FND-159 | high | FR-067-CON-1 has no test or symbol bound by the computed matrix | FR-067-CON-1 |
| FND-160 | high | FR-067-CON-5 has no test or symbol bound by the computed matrix | FR-067-CON-5 |
| FND-161 | high | FR-068-CON-2 has no test or symbol bound by the computed matrix | FR-068-CON-2 |
| FND-162 | high | FR-068-CON-4 has no test or symbol bound by the computed matrix | FR-068-CON-4 |
| FND-163 | high | FR-070-AC-11 has no test or symbol bound by the computed matrix | FR-070-AC-11 |
| FND-164 | high | FR-070-AC-12 has no test or symbol bound by the computed matrix | FR-070-AC-12 |
| FND-165 | high | FR-070-AC-13 has no test or symbol bound by the computed matrix | FR-070-AC-13 |
| FND-166 | high | FR-070-AC-14 has no test or symbol bound by the computed matrix | FR-070-AC-14 |
| FND-167 | high | FR-071-AC-9 has no test or symbol bound by the computed matrix | FR-071-AC-9 |
| FND-168 | high | FR-072-AC-6 has no test or symbol bound by the computed matrix | FR-072-AC-6 |
| FND-169 | high | FR-072-AC-9 has no test or symbol bound by the computed matrix | FR-072-AC-9 |
| FND-170 | high | FR-074-CON-1 has no test or symbol bound by the computed matrix | FR-074-CON-1 |
| FND-171 | high | FR-074-CON-2 has no test or symbol bound by the computed matrix | FR-074-CON-2 |
| FND-172 | high | FR-075-AC-16 has no test or symbol bound by the computed matrix | FR-075-AC-16 |
| FND-173 | high | FR-075-AC-17 has no test or symbol bound by the computed matrix | FR-075-AC-17 |
| FND-174 | high | FR-075-AC-18 has no test or symbol bound by the computed matrix | FR-075-AC-18 |
| FND-175 | high | FR-075-AC-19 has no test or symbol bound by the computed matrix | FR-075-AC-19 |
| FND-176 | high | FR-075-AC-20 has no test or symbol bound by the computed matrix | FR-075-AC-20 |
| FND-177 | high | FR-075-AC-21 has no test or symbol bound by the computed matrix | FR-075-AC-21 |
| FND-178 | high | FR-076-CON-1 has no test or symbol bound by the computed matrix | FR-076-CON-1 |
| FND-179 | high | FR-076-CON-2 has no test or symbol bound by the computed matrix | FR-076-CON-2 |
| FND-180 | high | FR-077-AC-1 has no test or symbol bound by the computed matrix | FR-077-AC-1 |
| FND-181 | high | FR-077-AC-3 has no test or symbol bound by the computed matrix | FR-077-AC-3 |
| FND-182 | high | FR-077-AC-4 has no test or symbol bound by the computed matrix | FR-077-AC-4 |
| FND-183 | high | FR-077-AC-5 has no test or symbol bound by the computed matrix | FR-077-AC-5 |
| FND-184 | high | FR-077-AC-6 has no test or symbol bound by the computed matrix | FR-077-AC-6 |
| FND-185 | high | FR-077-CON-3 has no test or symbol bound by the computed matrix | FR-077-CON-3 |
| FND-186 | high | FR-078-AC-1 has no test or symbol bound by the computed matrix | FR-078-AC-1 |
| FND-187 | high | FR-078-AC-2 has no test or symbol bound by the computed matrix | FR-078-AC-2 |
| FND-188 | high | FR-078-AC-3 has no test or symbol bound by the computed matrix | FR-078-AC-3 |
| FND-189 | high | FR-078-AC-4 has no test or symbol bound by the computed matrix | FR-078-AC-4 |
| FND-190 | high | FR-078-AC-5 has no test or symbol bound by the computed matrix | FR-078-AC-5 |
| FND-191 | high | FR-078-AC-6 has no test or symbol bound by the computed matrix | FR-078-AC-6 |
| FND-192 | high | FR-078-AC-7 has no test or symbol bound by the computed matrix | FR-078-AC-7 |
| FND-193 | high | FR-078-AC-8 has no test or symbol bound by the computed matrix | FR-078-AC-8 |
| FND-194 | high | FR-078-AC-9 has no test or symbol bound by the computed matrix | FR-078-AC-9 |
| FND-195 | high | FR-078-AC-10 has no test or symbol bound by the computed matrix | FR-078-AC-10 |
| FND-196 | high | FR-078-AC-11 has no test or symbol bound by the computed matrix | FR-078-AC-11 |
| FND-197 | high | FR-078-AC-12 has no test or symbol bound by the computed matrix | FR-078-AC-12 |
| FND-198 | high | FR-078-AC-13 has no test or symbol bound by the computed matrix | FR-078-AC-13 |
| FND-199 | high | FR-078-AC-14 has no test or symbol bound by the computed matrix | FR-078-AC-14 |
| FND-200 | high | FR-078-AC-15 has no test or symbol bound by the computed matrix | FR-078-AC-15 |
| FND-201 | high | FR-078-AC-16 has no test or symbol bound by the computed matrix | FR-078-AC-16 |
| FND-202 | high | FR-078-AC-17 has no test or symbol bound by the computed matrix | FR-078-AC-17 |
| FND-203 | high | FR-078-AC-18 has no test or symbol bound by the computed matrix | FR-078-AC-18 |
| FND-204 | high | FR-078-AC-19 has no test or symbol bound by the computed matrix | FR-078-AC-19 |
| FND-205 | high | FR-078-AC-20 has no test or symbol bound by the computed matrix | FR-078-AC-20 |
| FND-206 | high | FR-078-CON-2 has no test or symbol bound by the computed matrix | FR-078-CON-2 |
| FND-207 | high | FR-078-CON-3 has no test or symbol bound by the computed matrix | FR-078-CON-3 |
| FND-208 | high | FR-078-CON-4 has no test or symbol bound by the computed matrix | FR-078-CON-4 |
| FND-209 | high | NFR-001-AC-3 has no test or symbol bound by the computed matrix | NFR-001-AC-3 |
| FND-210 | high | NFR-001-M-1 has no test or symbol bound by the computed matrix | NFR-001-M-1 |
| FND-211 | high | NFR-001-M-2 has no test or symbol bound by the computed matrix | NFR-001-M-2 |
| FND-212 | high | NFR-001-M-3 has no test or symbol bound by the computed matrix | NFR-001-M-3 |
| FND-213 | high | NFR-002-AC-2 has no test or symbol bound by the computed matrix | NFR-002-AC-2 |
| FND-214 | high | NFR-002-AC-3 has no test or symbol bound by the computed matrix | NFR-002-AC-3 |
| FND-215 | high | NFR-002-M-1 has no test or symbol bound by the computed matrix | NFR-002-M-1 |
| FND-216 | high | NFR-002-M-2 has no test or symbol bound by the computed matrix | NFR-002-M-2 |
| FND-217 | high | NFR-002-M-3 has no test or symbol bound by the computed matrix | NFR-002-M-3 |
| FND-218 | high | NFR-002-M-4 has no test or symbol bound by the computed matrix | NFR-002-M-4 |
| FND-219 | high | NFR-003-AC-1 has no test or symbol bound by the computed matrix | NFR-003-AC-1 |
| FND-220 | high | NFR-003-AC-5 has no test or symbol bound by the computed matrix | NFR-003-AC-5 |
| FND-221 | high | NFR-003-M-1 has no test or symbol bound by the computed matrix | NFR-003-M-1 |
| FND-222 | high | NFR-003-M-2 has no test or symbol bound by the computed matrix | NFR-003-M-2 |
| FND-223 | high | NFR-003-M-3 has no test or symbol bound by the computed matrix | NFR-003-M-3 |
| FND-224 | high | NFR-003-M-4 has no test or symbol bound by the computed matrix | NFR-003-M-4 |
| FND-225 | high | NFR-003-M-5 has no test or symbol bound by the computed matrix | NFR-003-M-5 |
| FND-226 | high | NFR-004-AC-1 has no test or symbol bound by the computed matrix | NFR-004-AC-1 |
| FND-227 | high | NFR-004-AC-2 has no test or symbol bound by the computed matrix | NFR-004-AC-2 |
| FND-228 | high | NFR-004-M-1 has no test or symbol bound by the computed matrix | NFR-004-M-1 |
| FND-229 | high | NFR-004-M-2 has no test or symbol bound by the computed matrix | NFR-004-M-2 |
| FND-230 | high | NFR-004-M-3 has no test or symbol bound by the computed matrix | NFR-004-M-3 |
| FND-231 | high | NFR-005-AC-3 has no test or symbol bound by the computed matrix | NFR-005-AC-3 |
| FND-232 | high | NFR-005-M-3 has no test or symbol bound by the computed matrix | NFR-005-M-3 |
| FND-233 | high | NFR-005-M-4 has no test or symbol bound by the computed matrix | NFR-005-M-4 |
| FND-234 | high | NFR-006-AC-2 has no test or symbol bound by the computed matrix | NFR-006-AC-2 |
| FND-235 | high | NFR-006-M-1 has no test or symbol bound by the computed matrix | NFR-006-M-1 |
| FND-236 | high | NFR-006-M-2 has no test or symbol bound by the computed matrix | NFR-006-M-2 |
| FND-237 | high | NFR-006-M-3 has no test or symbol bound by the computed matrix | NFR-006-M-3 |
| FND-238 | high | NFR-007-AC-3 has no test or symbol bound by the computed matrix | NFR-007-AC-3 |
| FND-239 | high | NFR-007-AC-4 has no test or symbol bound by the computed matrix | NFR-007-AC-4 |
| FND-240 | high | NFR-007-M-1 has no test or symbol bound by the computed matrix | NFR-007-M-1 |
| FND-241 | high | NFR-007-M-2 has no test or symbol bound by the computed matrix | NFR-007-M-2 |
| FND-242 | high | NFR-007-M-3 has no test or symbol bound by the computed matrix | NFR-007-M-3 |
| FND-243 | high | NFR-007-M-4 has no test or symbol bound by the computed matrix | NFR-007-M-4 |
| FND-244 | high | NFR-009-AC-6 has no test or symbol bound by the computed matrix | NFR-009-AC-6 |
| FND-245 | high | NFR-009-AC-7 has no test or symbol bound by the computed matrix | NFR-009-AC-7 |
| FND-246 | high | NFR-009-AC-8 has no test or symbol bound by the computed matrix | NFR-009-AC-8 |
| FND-247 | high | NFR-009-AC-9 has no test or symbol bound by the computed matrix | NFR-009-AC-9 |
| FND-248 | high | NFR-009-M-1 has no test or symbol bound by the computed matrix | NFR-009-M-1 |
| FND-249 | high | NFR-009-M-2 has no test or symbol bound by the computed matrix | NFR-009-M-2 |
| FND-250 | high | NFR-009-M-3 has no test or symbol bound by the computed matrix | NFR-009-M-3 |
| FND-251 | high | NFR-010-AC-2 has no test or symbol bound by the computed matrix | NFR-010-AC-2 |
| FND-252 | high | NFR-010-M-1 has no test or symbol bound by the computed matrix | NFR-010-M-1 |
| FND-253 | high | NFR-010-M-2 has no test or symbol bound by the computed matrix | NFR-010-M-2 |
| FND-254 | high | NFR-011-AC-2 has no test or symbol bound by the computed matrix | NFR-011-AC-2 |
| FND-255 | high | NFR-011-AC-4 has no test or symbol bound by the computed matrix | NFR-011-AC-4 |
| FND-256 | high | NFR-011-M-2 has no test or symbol bound by the computed matrix | NFR-011-M-2 |
| FND-257 | high | NFR-011-M-3 has no test or symbol bound by the computed matrix | NFR-011-M-3 |
| FND-258 | high | NFR-012-AC-4 has no test or symbol bound by the computed matrix | NFR-012-AC-4 |
| FND-259 | high | NFR-012-M-2 has no test or symbol bound by the computed matrix | NFR-012-M-2 |
| FND-260 | high | NFR-013-AC-3 has no test or symbol bound by the computed matrix | NFR-013-AC-3 |
| FND-261 | high | NFR-013-M-1 has no test or symbol bound by the computed matrix | NFR-013-M-1 |
| FND-262 | high | NFR-013-M-2 has no test or symbol bound by the computed matrix | NFR-013-M-2 |
| FND-263 | high | NFR-013-M-3 has no test or symbol bound by the computed matrix | NFR-013-M-3 |
| FND-264 | high | NFR-014-AC-3 has no test or symbol bound by the computed matrix | NFR-014-AC-3 |
| FND-265 | high | NFR-014-M-1 has no test or symbol bound by the computed matrix | NFR-014-M-1 |
| FND-266 | high | NFR-014-M-4 has no test or symbol bound by the computed matrix | NFR-014-M-4 |
| FND-267 | high | NFR-015-AC-3 has no test or symbol bound by the computed matrix | NFR-015-AC-3 |
| FND-268 | high | NFR-015-AC-4 has no test or symbol bound by the computed matrix | NFR-015-AC-4 |
| FND-269 | high | NFR-015-M-1 has no test or symbol bound by the computed matrix | NFR-015-M-1 |
| FND-270 | high | NFR-015-M-2 has no test or symbol bound by the computed matrix | NFR-015-M-2 |
| FND-271 | high | NFR-015-M-3 has no test or symbol bound by the computed matrix | NFR-015-M-3 |
| FND-272 | high | NFR-015-M-4 has no test or symbol bound by the computed matrix | NFR-015-M-4 |
| FND-273 | high | NFR-016-AC-3 has no test or symbol bound by the computed matrix | NFR-016-AC-3 |
| FND-274 | high | NFR-016-AC-4 has no test or symbol bound by the computed matrix | NFR-016-AC-4 |
| FND-275 | high | NFR-016-M-1 has no test or symbol bound by the computed matrix | NFR-016-M-1 |
| FND-276 | high | NFR-016-M-2 has no test or symbol bound by the computed matrix | NFR-016-M-2 |
| FND-277 | high | NFR-016-M-3 has no test or symbol bound by the computed matrix | NFR-016-M-3 |
| FND-278 | high | NFR-017-AC-1 has no test or symbol bound by the computed matrix | NFR-017-AC-1 |
| FND-279 | high | NFR-017-M-1 has no test or symbol bound by the computed matrix | NFR-017-M-1 |
| FND-280 | high | NFR-017-M-2 has no test or symbol bound by the computed matrix | NFR-017-M-2 |
| FND-281 | high | NFR-017-M-3 has no test or symbol bound by the computed matrix | NFR-017-M-3 |
| FND-282 | high | NFR-018-AC-1 has no test or symbol bound by the computed matrix | NFR-018-AC-1 |
| FND-283 | high | NFR-018-AC-2 has no test or symbol bound by the computed matrix | NFR-018-AC-2 |
| FND-284 | high | NFR-018-AC-4 has no test or symbol bound by the computed matrix | NFR-018-AC-4 |
| FND-285 | high | NFR-018-M-1 has no test or symbol bound by the computed matrix | NFR-018-M-1 |
| FND-286 | high | NFR-018-M-2 has no test or symbol bound by the computed matrix | NFR-018-M-2 |
| FND-287 | high | NFR-019-M-1 has no test or symbol bound by the computed matrix | NFR-019-M-1 |
| FND-288 | high | NFR-019-M-2 has no test or symbol bound by the computed matrix | NFR-019-M-2 |
| FND-289 | high | NFR-020-AC-3 has no test or symbol bound by the computed matrix | NFR-020-AC-3 |
| FND-290 | high | NFR-020-M-2 has no test or symbol bound by the computed matrix | NFR-020-M-2 |
| FND-291 | high | NFR-020-M-3 has no test or symbol bound by the computed matrix | NFR-020-M-3 |
| FND-292 | high | NFR-021-M-3 has no test or symbol bound by the computed matrix | NFR-021-M-3 |
| FND-293 | high | NFR-021-M-4 has no test or symbol bound by the computed matrix | NFR-021-M-4 |
| FND-294 | medium | 47 trace-tagged symbols are not bound to a declared criterion; inspect ownership and tag model | quire coverage untracked_symbols |
| FND-295 | high | No run evidence is bound to 1015 criteria; matrix status reflects static traceability only | quoin matrix evidence_status |
| FND-296 | medium | Coverage reports 399 unbacked functional-coverage rows | spec/tests.md |

## Coverage

- Reconciliation: `quoin matrix` (quoin 0.29.2, quire 0.37.1) and `quire coverage` (quire 0.37.1); no run evidence read.
- Plan completion: not assessed
- Criteria: tagged 615 · untagged 293 · tagged-by-ignored-test 0 · method-without-symbol 107
- Evidence: no run evidence: 1015
- Coverage reconciliation: unbacked rows 399 · status lies 21 · no-symbol rows 44 · undeclared statuses 5 · untracked symbols 47 · shared trace IDs 81
- Evidence advice: 1015 recommendations; mismatch 122 · inconclusive 52 · uncatalogued 14. Recommendations are advisory, not evidence.
- Reverse audit: the source/test TODO/FIXME/XXX scan found no unfinished implementation in the changed path; untracked-symbol findings are summarized above.
- Semantic review: skipped

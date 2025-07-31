'use client'

import TokenvestingFeature from '@/components/tokenvesting/tokenvesting-feature'
import { EmployeeVestingList } from '@/components/tokenvesting/tokenvesting-employee-ui'

export default function TokenvestingPage() {
  return (
    <div className="space-y-8">
      <TokenvestingFeature />
      <EmployeeVestingList />
    </div>
  )
}
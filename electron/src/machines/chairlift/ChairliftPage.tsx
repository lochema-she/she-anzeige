import { Topbar } from "@/components/Topbar";
import { chairlift1SerialRoute } from "@/routes/routes";
import React from "react";

export function ChairliftPage() {
  const { serial } = chairlift1SerialRoute.useParams();
  return (
    <Topbar
      pathname={`/_sidebar/machines/chairlift1/${serial}`}
      items={[
        {
          link: "control",
          activeLink: "control",
          title: "Overview",
          icon: "lu:CableCar",
        },
      ]}
    />
  );
}

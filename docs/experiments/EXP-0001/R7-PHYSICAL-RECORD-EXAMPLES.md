# EXP-0001 R7 fictional physical-record examples

**Status:** documentation examples only; not fixtures, generated evidence, or authorization

All identities, URIs, digests, observations, and names below are deliberately fictional. Conformance is to the [closed ledger](R7-PHYSICAL-FIELD-LEDGER.md). Pretty printing in the first examples is for review; stored bytes would be their RFC 8785 JCS form with no newline.

## 1. Complete conforming environment record with explicit missing values

The explicit `missing` facts distinguish unavailable observations from zero; `not_applicable` marks concepts that do not apply. Empty arrays assert known empty collections.

```json
{
  "body": {
    "artifact_manifest": [
      {
        "artifact_id": "00000000-0000-4000-8000-000000000090",
        "byte_length": "0",
        "sha256": "0000000000000000000000000000000000000000000000000000000000000000",
        "uri": "file:/fictional/empty"
      }
    ],
    "authority_revisions": [
      {
        "path": "docs/experiments/EXP-0001/R7-PHYSICAL-FIELD-LEDGER.md",
        "sha256": "1111111111111111111111111111111111111111111111111111111111111111"
      }
    ],
    "baseline": {
      "state": "not_applicable"
    },
    "build": {
      "dependencies": [],
      "facts": [
        {
          "name": "command",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "compiler",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "compiler_flags",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "identity",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "linker",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "linker_flags",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "profile",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "rust_components",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "rust_host",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "rust_target",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "rust_toolchain",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "target",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "target_features",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        }
      ]
    },
    "capture": {
      "captured_at_utc_ns": "0",
      "mechanism": "fictional manual capture",
      "mechanism_version": {
        "state": "present",
        "value": "1"
      }
    },
    "clocks": [
      {
        "clock_class": "monotonic",
        "clock_id": "lifecycle",
        "conversion": {
          "state": "not_applicable"
        },
        "placement": "fictional harness boundary",
        "precision_ns": {
          "state": "present",
          "value": "1"
        },
        "resolution_ns": "1",
        "source": {
          "artifact_id": {
            "state": "not_applicable"
          },
          "mechanism": "fictional manual capture",
          "version": {
            "state": "present",
            "value": "1"
          }
        },
        "synchronization": {
          "state": "not_applicable"
        }
      }
    ],
    "configuration_refs": [
      {
        "configuration_kind": "platform_contract",
        "reference": {
          "artifact_id": "00000000-0000-4000-8000-000000000091",
          "byte_length": "0",
          "sha256": "2222222222222222222222222222222222222222222222222222222222222222",
          "uri": "file:/fictional/platform"
        }
      },
      {
        "configuration_kind": "workload",
        "reference": {
          "artifact_id": "00000000-0000-4000-8000-000000000092",
          "byte_length": "0",
          "sha256": "3333333333333333333333333333333333333333333333333333333333333333",
          "uri": "file:/fictional/workload"
        }
      }
    ],
    "cpu": {
      "facts": [
        {
          "name": "affinity",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "architecture",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "boost",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "features",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "frequency_governor",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "frequency_max_hz",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "frequency_min_hz",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "isolation",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "microcode",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "model",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "scaling_driver",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "smt",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        }
      ],
      "topology": [
        {
          "name": "benchmark_visible_cpus",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "dies",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "numa_nodes",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "physical_cores",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "sockets",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "threads",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        }
      ]
    },
    "data_locations": [
      {
        "location_role": "data",
        "permissions": "owner read/write",
        "pseudonym": "data-a",
        "stack_leaf": "medium-a"
      }
    ],
    "deviations": [],
    "durability_contract_ref": {
      "artifact_id": "00000000-0000-4000-8000-000000000091",
      "byte_length": "0",
      "sha256": "2222222222222222222222222222222222222222222222222222222222222222",
      "uri": "file:/fictional/platform"
    },
    "fault_apparatus": {
      "state": "not_applicable"
    },
    "host": {
      "execution_form": "bare_metal",
      "label": "fictional-host-a",
      "virtualization": {
        "state": "not_applicable"
      }
    },
    "instrumentation": [],
    "memory": {
      "capacity_bytes": {
        "source": {
          "artifact_id": {
            "state": "not_applicable"
          },
          "mechanism": "fictional manual capture",
          "version": {
            "state": "present",
            "value": "1"
          }
        },
        "value": {
          "state": "present",
          "value": "8589934592"
        }
      },
      "facts": [
        {
          "name": "channel_population",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "dimm_population",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "huge_pages",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "numa_policy",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "overcommit_policy",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "swap_devices",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "swap_observed_use",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "swap_policy",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "thp_policy",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "topology",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        }
      ],
      "limits": [
        {
          "name": "address_space",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "cgroup",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "job",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "locked",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "swap",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "visible",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        }
      ],
      "numa_nodes": [
        {
          "capacity_bytes": "8589934592",
          "node": 0
        }
      ],
      "speed_mt_s": {
        "source": {
          "artifact_id": {
            "state": "not_applicable"
          },
          "mechanism": "fictional manual capture",
          "version": {
            "state": "present",
            "value": "1"
          }
        },
        "value": {
          "reason": "fictional source did not report memory rate",
          "state": "missing"
        }
      }
    },
    "os": {
      "facts": [
        {
          "name": "background_activity",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "boot_parameters",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "distribution",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "image_release",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "kernel_architecture",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "kernel_build",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "kernel_release",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "name",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "power_policy",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "security_mitigations",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "thermal_state",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "version",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        }
      ]
    },
    "preparation": {
      "cache_initial_state": "fictional cold declaration",
      "cleanup_reuse_policy": "reset after each repetition",
      "preconditioning": "none",
      "subject_initial_state": "empty"
    },
    "record_producer": {
      "role": "documentation author",
      "tool": "manual fictional example",
      "version": {
        "state": "not_applicable"
      }
    },
    "redactions": [],
    "repository": {
      "commit": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
      "dirty_state": "clean",
      "patch_artifact_id": {
        "state": "not_applicable"
      },
      "submodules": []
    },
    "scheduler_security": {
      "facts": [
        {
          "name": "container_limits",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "io_limits",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "memory_limits",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "open_file_limits",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "privilege_posture",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "scheduler_class",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "scheduler_priority",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "scheduler_tunables",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "security_policy",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "thread_limits",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        }
      ]
    },
    "storage": {
      "block_sizes": [
        {
          "name": "filesystem_allocation",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "filesystem_io",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "logical_sector",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "physical_sector",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        }
      ],
      "facts": [
        {
          "name": "allocation",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "barrier_flush",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "cache_layers",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "controller",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "controller_battery",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "controller_cache",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "controller_firmware",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "controller_mode",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "device_capacity_bytes",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "device_firmware",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "device_interface",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "device_medium",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "device_model",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "discard",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "filesystem_creation_features",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "filesystem_type",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "filesystem_version",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "initial_capacity_use",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "mount_options",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "network_failure_domains",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "network_protocol",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "network_versions",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "power_loss_evidence",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "power_loss_protection",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "queue_count",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "queue_depth",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "queue_merge",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "queue_read_ahead_bytes",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "queue_scheduler",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "stable_device_pseudonym",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        },
        {
          "name": "volatile_layers",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "value": {
            "reason": "fictional example did not collect this field",
            "state": "missing"
          }
        }
      ],
      "free_space_bytes": {
        "source": {
          "artifact_id": {
            "state": "not_applicable"
          },
          "mechanism": "fictional manual capture",
          "version": {
            "state": "present",
            "value": "1"
          }
        },
        "value": {
          "reason": "fictional example deliberately omits observation",
          "state": "missing"
        }
      },
      "stack_path": [
        {
          "layer_ordinal": 0,
          "layer_type": "application",
          "pseudonym": "subject-a"
        },
        {
          "layer_ordinal": 1,
          "layer_type": "medium",
          "pseudonym": "medium-a"
        }
      ]
    }
  },
  "created_at_utc_ns": "0",
  "record_id": "00000000-0000-4000-8000-000000000001",
  "record_kind": "environment",
  "run_id": {
    "state": "not_applicable"
  },
  "schema_version": "EXP1-R7-JSON-JCS-1",
  "series_id": "00000000-0000-4000-8000-000000000002",
  "supersedes_record_id": {
    "state": "not_applicable"
  },
  "correction_reason": {
    "state": "not_applicable"
  }
}
```

## 2. Complete conforming raw-result record

This is a non-performance cleanup observation, so its empty operation population and zero-length throughput window are conforming and do not claim a benchmark measurement.

```json
{
  "body": {
    "ack_boundary": {
      "name": "cleanup-none",
      "source": {
        "artifact_id": {
          "state": "not_applicable"
        },
        "mechanism": "fictional manual capture",
        "version": {
          "state": "present",
          "value": "1"
        }
      }
    },
    "adapter_ref": {
      "artifact_id": "00000000-0000-4000-8000-000000000093",
      "byte_length": "0",
      "sha256": "4444444444444444444444444444444444444444444444444444444444444444",
      "uri": "file:/fictional/a93"
    },
    "allocations": {
      "bytes": {
        "method": "fictional cleanup observation",
        "scope": "process",
        "source": {
          "artifact_id": {
            "state": "not_applicable"
          },
          "mechanism": "fictional manual capture",
          "version": {
            "state": "present",
            "value": "1"
          }
        },
        "unit": "bytes",
        "value": {
          "reason": "not measured in fictional cleanup",
          "state": "not_collected"
        }
      },
      "count": {
        "method": "fictional cleanup observation",
        "scope": "process",
        "source": {
          "artifact_id": {
            "state": "not_applicable"
          },
          "mechanism": "fictional manual capture",
          "version": {
            "state": "present",
            "value": "1"
          }
        },
        "unit": "allocations",
        "value": {
          "reason": "not measured in fictional cleanup",
          "state": "not_collected"
        }
      }
    },
    "amplification": {
      "read": {
        "method": "fictional cleanup observation",
        "scope": "subject",
        "source": {
          "artifact_id": {
            "state": "not_applicable"
          },
          "mechanism": "fictional manual capture",
          "version": {
            "state": "present",
            "value": "1"
          }
        },
        "unit": "ratio",
        "value": {
          "reason": "not measured in fictional cleanup",
          "state": "not_collected"
        }
      },
      "space": {
        "method": "fictional cleanup observation",
        "scope": "subject",
        "source": {
          "artifact_id": {
            "state": "not_applicable"
          },
          "mechanism": "fictional manual capture",
          "version": {
            "state": "present",
            "value": "1"
          }
        },
        "unit": "ratio",
        "value": {
          "reason": "not measured in fictional cleanup",
          "state": "not_collected"
        }
      },
      "write": {
        "method": "fictional cleanup observation",
        "scope": "subject",
        "source": {
          "artifact_id": {
            "state": "not_applicable"
          },
          "mechanism": "fictional manual capture",
          "version": {
            "state": "present",
            "value": "1"
          }
        },
        "unit": "ratio",
        "value": {
          "reason": "not measured in fictional cleanup",
          "state": "not_collected"
        }
      }
    },
    "artifacts": {
      "artifacts": [
        {
          "artifact_id": "00000000-0000-4000-8000-000000000094",
          "byte_length": "0",
          "sha256": "6666666666666666666666666666666666666666666666666666666666666666",
          "uri": "file:/fictional/a94"
        }
      ],
      "manifest_ref": {
        "artifact_id": "00000000-0000-4000-8000-000000000096",
        "byte_length": "0",
        "sha256": "7777777777777777777777777777777777777777777777777777777777777777",
        "uri": "file:/fictional/a96"
      }
    },
    "background_work": [],
    "baseline_id": {
      "state": "not_applicable"
    },
    "canonical_status": "provisional",
    "configuration_refs": {
      "artifacts": [
        {
          "artifact_id": "00000000-0000-4000-8000-000000000097",
          "byte_length": "0",
          "sha256": "8888888888888888888888888888888888888888888888888888888888888888",
          "uri": "file:/fictional/a97"
        }
      ],
      "manifest_ref": {
        "artifact_id": "00000000-0000-4000-8000-000000000098",
        "byte_length": "0",
        "sha256": "9999999999999999999999999999999999999999999999999999999999999999",
        "uri": "file:/fictional/a98"
      }
    },
    "correctness": {
      "checks": [
        {
          "check_id": "cleanup-complete",
          "evidence_artifact_id": {
            "state": "present",
            "value": "00000000-0000-4000-8000-000000000094"
          },
          "message": {
            "state": "not_applicable"
          },
          "outcome": "pass"
        }
      ],
      "gate": "pass",
      "oracle_artifact_id": "00000000-0000-4000-8000-000000000094",
      "oracle_version": "fictional-1"
    },
    "cpu": {
      "system": {
        "method": "fictional cleanup observation",
        "scope": "process",
        "source": {
          "artifact_id": {
            "state": "not_applicable"
          },
          "mechanism": "fictional manual capture",
          "version": {
            "state": "present",
            "value": "1"
          }
        },
        "unit": "nanoseconds",
        "value": {
          "reason": "not measured in fictional cleanup",
          "state": "not_collected"
        }
      },
      "user": {
        "method": "fictional cleanup observation",
        "scope": "process",
        "source": {
          "artifact_id": {
            "state": "not_applicable"
          },
          "mechanism": "fictional manual capture",
          "version": {
            "state": "present",
            "value": "1"
          }
        },
        "unit": "nanoseconds",
        "value": {
          "reason": "not measured in fictional cleanup",
          "state": "not_collected"
        }
      },
      "wall": {
        "method": "fictional cleanup observation",
        "scope": "process",
        "source": {
          "artifact_id": {
            "state": "not_applicable"
          },
          "mechanism": "fictional manual capture",
          "version": {
            "state": "present",
            "value": "1"
          }
        },
        "unit": "nanoseconds",
        "value": {
          "reason": "not measured in fictional cleanup",
          "state": "not_collected"
        }
      }
    },
    "d_mode": "d0",
    "deviations": [],
    "encoded_bytes": [
      {
        "bytes": {
          "reason": "not measured in fictional cleanup",
          "state": "not_collected"
        },
        "domain": "complete_event",
        "method": "fictional cleanup observation"
      },
      {
        "bytes": {
          "reason": "not measured in fictional cleanup",
          "state": "not_collected"
        },
        "domain": "framing",
        "method": "fictional cleanup observation"
      },
      {
        "bytes": {
          "reason": "not measured in fictional cleanup",
          "state": "not_collected"
        },
        "domain": "integrity",
        "method": "fictional cleanup observation"
      }
    ],
    "environment_ref": {
      "record_id": "00000000-0000-4000-8000-000000000001",
      "record_sha256": "1111111111111111111111111111111111111111111111111111111111111111"
    },
    "equivalence": {
      "classification": "diagnostic",
      "conditions": [],
      "reasons": [
        "non-performance cleanup observation"
      ]
    },
    "errors": [],
    "execution_observations": {
      "backpressure": {
        "method": "fictional cleanup observation",
        "scope": "process",
        "source": {
          "artifact_id": {
            "state": "not_applicable"
          },
          "mechanism": "fictional manual capture",
          "version": {
            "state": "present",
            "value": "1"
          }
        },
        "unit": "events",
        "value": {
          "reason": "not measured in fictional cleanup",
          "state": "not_collected"
        }
      },
      "checkpoints": {
        "method": "fictional cleanup observation",
        "scope": "process",
        "source": {
          "artifact_id": {
            "state": "not_applicable"
          },
          "mechanism": "fictional manual capture",
          "version": {
            "state": "present",
            "value": "1"
          }
        },
        "unit": "events",
        "value": {
          "reason": "not measured in fictional cleanup",
          "state": "not_collected"
        }
      },
      "compactions": {
        "method": "fictional cleanup observation",
        "scope": "process",
        "source": {
          "artifact_id": {
            "state": "not_applicable"
          },
          "mechanism": "fictional manual capture",
          "version": {
            "state": "present",
            "value": "1"
          }
        },
        "unit": "events",
        "value": {
          "reason": "not measured in fictional cleanup",
          "state": "not_collected"
        }
      },
      "errors": {
        "method": "fictional cleanup observation",
        "scope": "process",
        "source": {
          "artifact_id": {
            "state": "not_applicable"
          },
          "mechanism": "fictional manual capture",
          "version": {
            "state": "present",
            "value": "1"
          }
        },
        "unit": "events",
        "value": {
          "reason": "not measured in fictional cleanup",
          "state": "not_collected"
        }
      },
      "flushes": {
        "method": "fictional cleanup observation",
        "scope": "process",
        "source": {
          "artifact_id": {
            "state": "not_applicable"
          },
          "mechanism": "fictional manual capture",
          "version": {
            "state": "present",
            "value": "1"
          }
        },
        "unit": "events",
        "value": {
          "reason": "not measured in fictional cleanup",
          "state": "not_collected"
        }
      },
      "partial_writes": {
        "method": "fictional cleanup observation",
        "scope": "process",
        "source": {
          "artifact_id": {
            "state": "not_applicable"
          },
          "mechanism": "fictional manual capture",
          "version": {
            "state": "present",
            "value": "1"
          }
        },
        "unit": "events",
        "value": {
          "reason": "not measured in fictional cleanup",
          "state": "not_collected"
        }
      },
      "retries": {
        "method": "fictional cleanup observation",
        "scope": "process",
        "source": {
          "artifact_id": {
            "state": "not_applicable"
          },
          "mechanism": "fictional manual capture",
          "version": {
            "state": "present",
            "value": "1"
          }
        },
        "unit": "events",
        "value": {
          "reason": "not measured in fictional cleanup",
          "state": "not_collected"
        }
      },
      "stalls": {
        "method": "fictional cleanup observation",
        "scope": "process",
        "source": {
          "artifact_id": {
            "state": "not_applicable"
          },
          "mechanism": "fictional manual capture",
          "version": {
            "state": "present",
            "value": "1"
          }
        },
        "unit": "events",
        "value": {
          "reason": "not measured in fictional cleanup",
          "state": "not_collected"
        }
      }
    },
    "experiment_ref": "EXP-0001",
    "fault_contract": {
      "state": "not_applicable"
    },
    "hypothesis_refs": [
      "HYP-0001"
    ],
    "interval": {
      "clock_id": "fictional-monotonic",
      "elapsed_ns": "0",
      "end": {
        "state": "present",
        "value": "0"
      },
      "method": "fictional cleanup endpoints",
      "precision_ns": {
        "state": "present",
        "value": "1"
      },
      "source": {
        "artifact_id": {
          "state": "not_applicable"
        },
        "mechanism": "fictional manual capture",
        "version": {
          "state": "present",
          "value": "1"
        }
      },
      "start": {
        "state": "present",
        "value": "0"
      },
      "time_domain": "monotonic"
    },
    "io": [
      {
        "bytes": {
          "method": "fictional cleanup observation",
          "scope": "application",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "unit": "bytes",
          "value": {
            "reason": "not measured in fictional cleanup",
            "state": "not_collected"
          }
        },
        "count": {
          "method": "fictional cleanup observation",
          "scope": "application",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "unit": "operations",
          "value": {
            "reason": "not measured in fictional cleanup",
            "state": "not_collected"
          }
        },
        "layer": "application",
        "operation": "read"
      },
      {
        "bytes": {
          "method": "fictional cleanup observation",
          "scope": "application",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "unit": "bytes",
          "value": {
            "reason": "not measured in fictional cleanup",
            "state": "not_collected"
          }
        },
        "count": {
          "method": "fictional cleanup observation",
          "scope": "application",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "unit": "operations",
          "value": {
            "reason": "not measured in fictional cleanup",
            "state": "not_collected"
          }
        },
        "layer": "application",
        "operation": "write"
      },
      {
        "bytes": {
          "method": "fictional cleanup observation",
          "scope": "vfs",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "unit": "bytes",
          "value": {
            "reason": "not measured in fictional cleanup",
            "state": "not_collected"
          }
        },
        "count": {
          "method": "fictional cleanup observation",
          "scope": "vfs",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "unit": "operations",
          "value": {
            "reason": "not measured in fictional cleanup",
            "state": "not_collected"
          }
        },
        "layer": "vfs",
        "operation": "read"
      },
      {
        "bytes": {
          "method": "fictional cleanup observation",
          "scope": "vfs",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "unit": "bytes",
          "value": {
            "reason": "not measured in fictional cleanup",
            "state": "not_collected"
          }
        },
        "count": {
          "method": "fictional cleanup observation",
          "scope": "vfs",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "unit": "operations",
          "value": {
            "reason": "not measured in fictional cleanup",
            "state": "not_collected"
          }
        },
        "layer": "vfs",
        "operation": "write"
      },
      {
        "bytes": {
          "method": "fictional cleanup observation",
          "scope": "filesystem",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "unit": "bytes",
          "value": {
            "reason": "not measured in fictional cleanup",
            "state": "not_collected"
          }
        },
        "count": {
          "method": "fictional cleanup observation",
          "scope": "filesystem",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "unit": "operations",
          "value": {
            "reason": "not measured in fictional cleanup",
            "state": "not_collected"
          }
        },
        "layer": "filesystem",
        "operation": "read"
      },
      {
        "bytes": {
          "method": "fictional cleanup observation",
          "scope": "filesystem",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "unit": "bytes",
          "value": {
            "reason": "not measured in fictional cleanup",
            "state": "not_collected"
          }
        },
        "count": {
          "method": "fictional cleanup observation",
          "scope": "filesystem",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "unit": "operations",
          "value": {
            "reason": "not measured in fictional cleanup",
            "state": "not_collected"
          }
        },
        "layer": "filesystem",
        "operation": "write"
      },
      {
        "bytes": {
          "method": "fictional cleanup observation",
          "scope": "block",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "unit": "bytes",
          "value": {
            "reason": "not measured in fictional cleanup",
            "state": "not_collected"
          }
        },
        "count": {
          "method": "fictional cleanup observation",
          "scope": "block",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "unit": "operations",
          "value": {
            "reason": "not measured in fictional cleanup",
            "state": "not_collected"
          }
        },
        "layer": "block",
        "operation": "read"
      },
      {
        "bytes": {
          "method": "fictional cleanup observation",
          "scope": "block",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "unit": "bytes",
          "value": {
            "reason": "not measured in fictional cleanup",
            "state": "not_collected"
          }
        },
        "count": {
          "method": "fictional cleanup observation",
          "scope": "block",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "unit": "operations",
          "value": {
            "reason": "not measured in fictional cleanup",
            "state": "not_collected"
          }
        },
        "layer": "block",
        "operation": "write"
      },
      {
        "bytes": {
          "method": "fictional cleanup observation",
          "scope": "device",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "unit": "bytes",
          "value": {
            "reason": "not measured in fictional cleanup",
            "state": "not_collected"
          }
        },
        "count": {
          "method": "fictional cleanup observation",
          "scope": "device",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "unit": "operations",
          "value": {
            "reason": "not measured in fictional cleanup",
            "state": "not_collected"
          }
        },
        "layer": "device",
        "operation": "read"
      },
      {
        "bytes": {
          "method": "fictional cleanup observation",
          "scope": "device",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "unit": "bytes",
          "value": {
            "reason": "not measured in fictional cleanup",
            "state": "not_collected"
          }
        },
        "count": {
          "method": "fictional cleanup observation",
          "scope": "device",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "unit": "operations",
          "value": {
            "reason": "not measured in fictional cleanup",
            "state": "not_collected"
          }
        },
        "layer": "device",
        "operation": "write"
      },
      {
        "bytes": {
          "method": "fictional cleanup observation",
          "scope": "other",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "unit": "bytes",
          "value": {
            "reason": "not measured in fictional cleanup",
            "state": "not_collected"
          }
        },
        "count": {
          "method": "fictional cleanup observation",
          "scope": "other",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "unit": "operations",
          "value": {
            "reason": "not measured in fictional cleanup",
            "state": "not_collected"
          }
        },
        "layer": "other",
        "operation": "read"
      },
      {
        "bytes": {
          "method": "fictional cleanup observation",
          "scope": "other",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "unit": "bytes",
          "value": {
            "reason": "not measured in fictional cleanup",
            "state": "not_collected"
          }
        },
        "count": {
          "method": "fictional cleanup observation",
          "scope": "other",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "unit": "operations",
          "value": {
            "reason": "not measured in fictional cleanup",
            "state": "not_collected"
          }
        },
        "layer": "other",
        "operation": "write"
      }
    ],
    "latency": {
      "algorithm": "none for non-performance cleanup",
      "evidence": {
        "reason": "not measured in fictional cleanup",
        "state": "not_collected"
      },
      "interval": {
        "clock_id": "fictional-monotonic",
        "elapsed_ns": "0",
        "end": {
          "state": "present",
          "value": "0"
        },
        "method": "fictional cleanup endpoints",
        "precision_ns": {
          "state": "present",
          "value": "1"
        },
        "source": {
          "artifact_id": {
            "state": "not_applicable"
          },
          "mechanism": "fictional manual capture",
          "version": {
            "state": "present",
            "value": "1"
          }
        },
        "start": {
          "state": "present",
          "value": "0"
        },
        "time_domain": "monotonic"
      },
      "loss": {
        "state": "present",
        "value": {
          "lost": "0",
          "reason": "no samples expected"
        }
      },
      "method": "fictional cleanup observation",
      "population": "0",
      "rounding": "no rounding performed",
      "unit": "nanoseconds"
    },
    "lifecycle_interval": {
      "clock_id": "fictional-monotonic",
      "elapsed_ns": "0",
      "end": {
        "state": "present",
        "value": "0"
      },
      "method": "fictional cleanup endpoints",
      "precision_ns": {
        "state": "present",
        "value": "1"
      },
      "source": {
        "artifact_id": {
          "state": "not_applicable"
        },
        "mechanism": "fictional manual capture",
        "version": {
          "state": "present",
          "value": "1"
        }
      },
      "start": {
        "state": "present",
        "value": "0"
      },
      "time_domain": "monotonic"
    },
    "logical_bytes": [
      {
        "bytes": {
          "reason": "not measured in fictional cleanup",
          "state": "not_collected"
        },
        "domain": "envelope",
        "method": "fictional cleanup observation"
      },
      {
        "bytes": {
          "reason": "not measured in fictional cleanup",
          "state": "not_collected"
        },
        "domain": "key",
        "method": "fictional cleanup observation"
      },
      {
        "bytes": {
          "reason": "not measured in fictional cleanup",
          "state": "not_collected"
        },
        "domain": "payload",
        "method": "fictional cleanup observation"
      },
      {
        "bytes": {
          "reason": "not measured in fictional cleanup",
          "state": "not_collected"
        },
        "domain": "value",
        "method": "fictional cleanup observation"
      }
    ],
    "memory": {
      "cache": {
        "method": "fictional cleanup observation",
        "scope": "process",
        "source": {
          "artifact_id": {
            "state": "not_applicable"
          },
          "mechanism": "fictional manual capture",
          "version": {
            "state": "present",
            "value": "1"
          }
        },
        "unit": "bytes",
        "value": {
          "reason": "not measured in fictional cleanup",
          "state": "not_collected"
        }
      },
      "peak_resident": {
        "method": "fictional cleanup observation",
        "scope": "process",
        "source": {
          "artifact_id": {
            "state": "not_applicable"
          },
          "mechanism": "fictional manual capture",
          "version": {
            "state": "present",
            "value": "1"
          }
        },
        "unit": "bytes",
        "value": {
          "reason": "not measured in fictional cleanup",
          "state": "not_collected"
        }
      },
      "resident": {
        "method": "fictional cleanup observation",
        "scope": "process",
        "source": {
          "artifact_id": {
            "state": "not_applicable"
          },
          "mechanism": "fictional manual capture",
          "version": {
            "state": "present",
            "value": "1"
          }
        },
        "unit": "bytes",
        "value": {
          "reason": "not measured in fictional cleanup",
          "state": "not_collected"
        }
      },
      "virtual": {
        "method": "fictional cleanup observation",
        "scope": "process",
        "source": {
          "artifact_id": {
            "state": "not_applicable"
          },
          "mechanism": "fictional manual capture",
          "version": {
            "state": "present",
            "value": "1"
          }
        },
        "unit": "bytes",
        "value": {
          "reason": "not measured in fictional cleanup",
          "state": "not_collected"
        }
      }
    },
    "operation_counts": {
      "accepted": {
        "method": "fictional cleanup counter",
        "unit": "operations",
        "value": {
          "state": "present",
          "value": "0"
        }
      },
      "acknowledged": {
        "method": "fictional cleanup counter",
        "unit": "operations",
        "value": {
          "state": "present",
          "value": "0"
        }
      },
      "attempted": {
        "method": "fictional cleanup counter",
        "unit": "operations",
        "value": {
          "state": "present",
          "value": "0"
        }
      },
      "committed": {
        "method": "fictional cleanup counter",
        "unit": "operations",
        "value": {
          "state": "present",
          "value": "0"
        }
      },
      "corrupt": {
        "method": "fictional cleanup counter",
        "unit": "operations",
        "value": {
          "state": "present",
          "value": "0"
        }
      },
      "failed": {
        "method": "fictional cleanup counter",
        "unit": "operations",
        "value": {
          "state": "present",
          "value": "0"
        }
      },
      "missing": {
        "method": "fictional cleanup counter",
        "unit": "operations",
        "value": {
          "state": "present",
          "value": "0"
        }
      },
      "provisional": {
        "method": "fictional cleanup counter",
        "unit": "operations",
        "value": {
          "state": "present",
          "value": "0"
        }
      },
      "recovered": {
        "method": "fictional cleanup counter",
        "unit": "operations",
        "value": {
          "state": "present",
          "value": "0"
        }
      },
      "rejected": {
        "method": "fictional cleanup counter",
        "unit": "operations",
        "value": {
          "state": "present",
          "value": "0"
        }
      },
      "uncertain": {
        "method": "fictional cleanup counter",
        "unit": "operations",
        "value": {
          "state": "present",
          "value": "0"
        }
      }
    },
    "operations": [],
    "phase": {
      "name": "cleanup",
      "observation_role": "non_performance"
    },
    "physical_bytes": [
      {
        "bytes": {
          "reason": "not measured in fictional cleanup",
          "state": "not_collected"
        },
        "domain": "checkpoint",
        "method": "fictional cleanup observation"
      },
      {
        "bytes": {
          "reason": "not measured in fictional cleanup",
          "state": "not_collected"
        },
        "domain": "compaction",
        "method": "fictional cleanup observation"
      },
      {
        "bytes": {
          "reason": "not measured in fictional cleanup",
          "state": "not_collected"
        },
        "domain": "database",
        "method": "fictional cleanup observation"
      },
      {
        "bytes": {
          "reason": "not measured in fictional cleanup",
          "state": "not_collected"
        },
        "domain": "manifest",
        "method": "fictional cleanup observation"
      },
      {
        "bytes": {
          "reason": "not measured in fictional cleanup",
          "state": "not_collected"
        },
        "domain": "other",
        "method": "fictional cleanup observation"
      },
      {
        "bytes": {
          "reason": "not measured in fictional cleanup",
          "state": "not_collected"
        },
        "domain": "read",
        "method": "fictional cleanup observation"
      },
      {
        "bytes": {
          "reason": "not measured in fictional cleanup",
          "state": "not_collected"
        },
        "domain": "requested_io",
        "method": "fictional cleanup observation"
      },
      {
        "bytes": {
          "reason": "not measured in fictional cleanup",
          "state": "not_collected"
        },
        "domain": "sst",
        "method": "fictional cleanup observation"
      },
      {
        "bytes": {
          "reason": "not measured in fictional cleanup",
          "state": "not_collected"
        },
        "domain": "synchronized",
        "method": "fictional cleanup observation"
      },
      {
        "bytes": {
          "reason": "not measured in fictional cleanup",
          "state": "not_collected"
        },
        "domain": "temporary",
        "method": "fictional cleanup observation"
      },
      {
        "bytes": {
          "reason": "not measured in fictional cleanup",
          "state": "not_collected"
        },
        "domain": "wal",
        "method": "fictional cleanup observation"
      },
      {
        "bytes": {
          "reason": "not measured in fictional cleanup",
          "state": "not_collected"
        },
        "domain": "written",
        "method": "fictional cleanup observation"
      }
    ],
    "platform_contract_ref": {
      "artifact_id": "00000000-0000-4000-8000-000000000091",
      "byte_length": "0",
      "sha256": "2222222222222222222222222222222222222222222222222222222222222222",
      "uri": "file:/fictional/a91"
    },
    "producer_record": {
      "artifact_id": "00000000-0000-4000-8000-000000000095",
      "byte_length": "0",
      "sha256": "5555555555555555555555555555555555555555555555555555555555555555",
      "uri": "file:/fictional/a95"
    },
    "profile_id": "fictional-cleanup",
    "provenance": {
      "edge_artifact_ref": {
        "artifact_id": "00000000-0000-4000-8000-000000000099",
        "byte_length": "0",
        "sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "uri": "file:/fictional/a99"
      },
      "endpoint_artifact_ids": [
        "00000000-0000-4000-8000-000000000003",
        "00000000-0000-4000-8000-000000000091",
        "00000000-0000-4000-8000-000000000092",
        "00000000-0000-4000-8000-000000000093",
        "00000000-0000-4000-8000-000000000094",
        "00000000-0000-4000-8000-000000000095",
        "00000000-0000-4000-8000-000000000096",
        "00000000-0000-4000-8000-000000000097",
        "00000000-0000-4000-8000-000000000098",
        "00000000-0000-4000-8000-000000000099"
      ]
    },
    "recovery": {
      "state": "not_applicable"
    },
    "repetition_id": "rep-0",
    "repository": {
      "commit": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
      "dirty_state": "clean",
      "patch_artifact_id": {
        "state": "not_applicable"
      }
    },
    "requirement_refs": [
      "REQ-014"
    ],
    "resource_measurements": [],
    "result_classification": {
      "labels": [
        "valid",
        "diagnostic"
      ],
      "reasons": [
        "fictional cleanup only"
      ]
    },
    "sample_id": "cleanup-sample-0",
    "sample_population": {
      "included": "0",
      "lost": "0",
      "omission_reason": {
        "state": "not_applicable"
      },
      "total": "0"
    },
    "subject_id": "fictional-subject",
    "synchronization": {
      "completed": {
        "method": "fictional cleanup observation",
        "scope": "process",
        "source": {
          "artifact_id": {
            "state": "not_applicable"
          },
          "mechanism": "fictional manual capture",
          "version": {
            "state": "present",
            "value": "1"
          }
        },
        "unit": "operations",
        "value": {
          "reason": "not measured in fictional cleanup",
          "state": "not_collected"
        }
      },
      "failed": {
        "method": "fictional cleanup observation",
        "scope": "process",
        "source": {
          "artifact_id": {
            "state": "not_applicable"
          },
          "mechanism": "fictional manual capture",
          "version": {
            "state": "present",
            "value": "1"
          }
        },
        "unit": "operations",
        "value": {
          "reason": "not measured in fictional cleanup",
          "state": "not_collected"
        }
      },
      "group_scope": {
        "state": "not_applicable"
      },
      "primitive": "none for cleanup",
      "requested": {
        "method": "fictional cleanup observation",
        "scope": "process",
        "source": {
          "artifact_id": {
            "state": "not_applicable"
          },
          "mechanism": "fictional manual capture",
          "version": {
            "state": "present",
            "value": "1"
          }
        },
        "unit": "operations",
        "value": {
          "reason": "not measured in fictional cleanup",
          "state": "not_collected"
        }
      },
      "wait": {
        "method": "fictional cleanup observation",
        "scope": "process",
        "source": {
          "artifact_id": {
            "state": "not_applicable"
          },
          "mechanism": "fictional manual capture",
          "version": {
            "state": "present",
            "value": "1"
          }
        },
        "unit": "nanoseconds",
        "value": {
          "reason": "not measured in fictional cleanup",
          "state": "not_collected"
        }
      }
    },
    "throughput": {
      "bytes": {
        "denominator_ns": "0",
        "interval": {
          "clock_id": "fictional-monotonic",
          "elapsed_ns": "0",
          "end": {
            "state": "present",
            "value": "0"
          },
          "method": "fictional cleanup endpoints",
          "precision_ns": {
            "state": "present",
            "value": "1"
          },
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "start": {
            "state": "present",
            "value": "0"
          },
          "time_domain": "monotonic"
        },
        "method": "fictional non-performance rate",
        "numerator": "0",
        "unit": "bytes_per_second",
        "value": {
          "reason": "rate not applicable to cleanup",
          "state": "not_collected"
        }
      },
      "operations": {
        "denominator_ns": "0",
        "interval": {
          "clock_id": "fictional-monotonic",
          "elapsed_ns": "0",
          "end": {
            "state": "present",
            "value": "0"
          },
          "method": "fictional cleanup endpoints",
          "precision_ns": {
            "state": "present",
            "value": "1"
          },
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "start": {
            "state": "present",
            "value": "0"
          },
          "time_domain": "monotonic"
        },
        "method": "fictional non-performance rate",
        "numerator": "0",
        "unit": "operations_per_second",
        "value": {
          "reason": "rate not applicable to cleanup",
          "state": "not_collected"
        }
      }
    },
    "time_meanings": {
      "durability": {
        "reason": "time meaning not applicable to cleanup",
        "state": "not_collected"
      },
      "effective": {
        "reason": "time meaning not applicable to cleanup",
        "state": "not_collected"
      },
      "observation": {
        "reason": "time meaning not applicable to cleanup",
        "state": "not_collected"
      },
      "system_acceptance": {
        "reason": "time meaning not applicable to cleanup",
        "state": "not_collected"
      }
    },
    "validation": {
      "findings": [
        "fictional example is not repository evidence"
      ],
      "integrity": [],
      "status": "not_validated",
      "validated_at_utc_ns": {
        "reason": "fictional example not validated",
        "state": "not_collected"
      },
      "validator_configuration_ref": {
        "reason": "fictional example not validated",
        "state": "not_collected"
      },
      "validator_identity": {
        "reason": "fictional example not validated",
        "state": "not_collected"
      },
      "validator_version": {
        "reason": "fictional example not validated",
        "state": "not_collected"
      }
    },
    "visibility": {
      "first_visible_monotonic_ns": {
        "state": "not_applicable"
      },
      "probe": "not applicable during cleanup",
      "status": "not_observed"
    },
    "workload_ref": {
      "artifact_id": "00000000-0000-4000-8000-000000000092",
      "byte_length": "0",
      "sha256": "3333333333333333333333333333333333333333333333333333333333333333",
      "uri": "file:/fictional/a92"
    }
  },
  "created_at_utc_ns": "1",
  "record_id": "00000000-0000-4000-8000-000000000003",
  "record_kind": "raw_result",
  "run_id": {
    "state": "present",
    "value": "00000000-0000-4000-8000-000000000004"
  },
  "schema_version": "EXP1-R7-JSON-JCS-1",
  "series_id": "00000000-0000-4000-8000-000000000002",
  "supersedes_record_id": {
    "state": "not_applicable"
  },
  "correction_reason": {
    "state": "not_applicable"
  }
}
```

## 3. Immutable correction and supersession

The following complete record corrects the preceding raw result. It has a new identity, retains the same run and series, and names the old identity. Publication additionally requires a `corrects` provenance edge between their artifacts; neither old bytes nor identity changes.

```json
{
  "body": {
    "ack_boundary": {
      "name": "cleanup-none",
      "source": {
        "artifact_id": {
          "state": "not_applicable"
        },
        "mechanism": "fictional manual capture",
        "version": {
          "state": "present",
          "value": "1"
        }
      }
    },
    "adapter_ref": {
      "artifact_id": "00000000-0000-4000-8000-000000000093",
      "byte_length": "0",
      "sha256": "4444444444444444444444444444444444444444444444444444444444444444",
      "uri": "file:/fictional/a93"
    },
    "allocations": {
      "bytes": {
        "method": "fictional cleanup observation",
        "scope": "process",
        "source": {
          "artifact_id": {
            "state": "not_applicable"
          },
          "mechanism": "fictional manual capture",
          "version": {
            "state": "present",
            "value": "1"
          }
        },
        "unit": "bytes",
        "value": {
          "reason": "not measured in fictional cleanup",
          "state": "not_collected"
        }
      },
      "count": {
        "method": "fictional cleanup observation",
        "scope": "process",
        "source": {
          "artifact_id": {
            "state": "not_applicable"
          },
          "mechanism": "fictional manual capture",
          "version": {
            "state": "present",
            "value": "1"
          }
        },
        "unit": "allocations",
        "value": {
          "reason": "not measured in fictional cleanup",
          "state": "not_collected"
        }
      }
    },
    "amplification": {
      "read": {
        "method": "fictional cleanup observation",
        "scope": "subject",
        "source": {
          "artifact_id": {
            "state": "not_applicable"
          },
          "mechanism": "fictional manual capture",
          "version": {
            "state": "present",
            "value": "1"
          }
        },
        "unit": "ratio",
        "value": {
          "reason": "not measured in fictional cleanup",
          "state": "not_collected"
        }
      },
      "space": {
        "method": "fictional cleanup observation",
        "scope": "subject",
        "source": {
          "artifact_id": {
            "state": "not_applicable"
          },
          "mechanism": "fictional manual capture",
          "version": {
            "state": "present",
            "value": "1"
          }
        },
        "unit": "ratio",
        "value": {
          "reason": "not measured in fictional cleanup",
          "state": "not_collected"
        }
      },
      "write": {
        "method": "fictional cleanup observation",
        "scope": "subject",
        "source": {
          "artifact_id": {
            "state": "not_applicable"
          },
          "mechanism": "fictional manual capture",
          "version": {
            "state": "present",
            "value": "1"
          }
        },
        "unit": "ratio",
        "value": {
          "reason": "not measured in fictional cleanup",
          "state": "not_collected"
        }
      }
    },
    "artifacts": {
      "artifacts": [
        {
          "artifact_id": "00000000-0000-4000-8000-000000000094",
          "byte_length": "0",
          "sha256": "6666666666666666666666666666666666666666666666666666666666666666",
          "uri": "file:/fictional/a94"
        }
      ],
      "manifest_ref": {
        "artifact_id": "00000000-0000-4000-8000-000000000096",
        "byte_length": "0",
        "sha256": "7777777777777777777777777777777777777777777777777777777777777777",
        "uri": "file:/fictional/a96"
      }
    },
    "background_work": [],
    "baseline_id": {
      "state": "not_applicable"
    },
    "canonical_status": "provisional",
    "configuration_refs": {
      "artifacts": [
        {
          "artifact_id": "00000000-0000-4000-8000-000000000097",
          "byte_length": "0",
          "sha256": "8888888888888888888888888888888888888888888888888888888888888888",
          "uri": "file:/fictional/a97"
        }
      ],
      "manifest_ref": {
        "artifact_id": "00000000-0000-4000-8000-000000000098",
        "byte_length": "0",
        "sha256": "9999999999999999999999999999999999999999999999999999999999999999",
        "uri": "file:/fictional/a98"
      }
    },
    "correctness": {
      "checks": [
        {
          "check_id": "cleanup-complete",
          "evidence_artifact_id": {
            "state": "present",
            "value": "00000000-0000-4000-8000-000000000094"
          },
          "message": {
            "state": "not_applicable"
          },
          "outcome": "pass"
        }
      ],
      "gate": "pass",
      "oracle_artifact_id": "00000000-0000-4000-8000-000000000094",
      "oracle_version": "fictional-1"
    },
    "cpu": {
      "system": {
        "method": "fictional cleanup observation",
        "scope": "process",
        "source": {
          "artifact_id": {
            "state": "not_applicable"
          },
          "mechanism": "fictional manual capture",
          "version": {
            "state": "present",
            "value": "1"
          }
        },
        "unit": "nanoseconds",
        "value": {
          "reason": "not measured in fictional cleanup",
          "state": "not_collected"
        }
      },
      "user": {
        "method": "fictional cleanup observation",
        "scope": "process",
        "source": {
          "artifact_id": {
            "state": "not_applicable"
          },
          "mechanism": "fictional manual capture",
          "version": {
            "state": "present",
            "value": "1"
          }
        },
        "unit": "nanoseconds",
        "value": {
          "reason": "not measured in fictional cleanup",
          "state": "not_collected"
        }
      },
      "wall": {
        "method": "fictional cleanup observation",
        "scope": "process",
        "source": {
          "artifact_id": {
            "state": "not_applicable"
          },
          "mechanism": "fictional manual capture",
          "version": {
            "state": "present",
            "value": "1"
          }
        },
        "unit": "nanoseconds",
        "value": {
          "reason": "not measured in fictional cleanup",
          "state": "not_collected"
        }
      }
    },
    "d_mode": "d0",
    "deviations": [],
    "encoded_bytes": [
      {
        "bytes": {
          "reason": "not measured in fictional cleanup",
          "state": "not_collected"
        },
        "domain": "complete_event",
        "method": "fictional cleanup observation"
      },
      {
        "bytes": {
          "reason": "not measured in fictional cleanup",
          "state": "not_collected"
        },
        "domain": "framing",
        "method": "fictional cleanup observation"
      },
      {
        "bytes": {
          "reason": "not measured in fictional cleanup",
          "state": "not_collected"
        },
        "domain": "integrity",
        "method": "fictional cleanup observation"
      }
    ],
    "environment_ref": {
      "record_id": "00000000-0000-4000-8000-000000000001",
      "record_sha256": "1111111111111111111111111111111111111111111111111111111111111111"
    },
    "equivalence": {
      "classification": "diagnostic",
      "conditions": [],
      "reasons": [
        "non-performance cleanup observation"
      ]
    },
    "errors": [],
    "execution_observations": {
      "backpressure": {
        "method": "fictional cleanup observation",
        "scope": "process",
        "source": {
          "artifact_id": {
            "state": "not_applicable"
          },
          "mechanism": "fictional manual capture",
          "version": {
            "state": "present",
            "value": "1"
          }
        },
        "unit": "events",
        "value": {
          "reason": "not measured in fictional cleanup",
          "state": "not_collected"
        }
      },
      "checkpoints": {
        "method": "fictional cleanup observation",
        "scope": "process",
        "source": {
          "artifact_id": {
            "state": "not_applicable"
          },
          "mechanism": "fictional manual capture",
          "version": {
            "state": "present",
            "value": "1"
          }
        },
        "unit": "events",
        "value": {
          "reason": "not measured in fictional cleanup",
          "state": "not_collected"
        }
      },
      "compactions": {
        "method": "fictional cleanup observation",
        "scope": "process",
        "source": {
          "artifact_id": {
            "state": "not_applicable"
          },
          "mechanism": "fictional manual capture",
          "version": {
            "state": "present",
            "value": "1"
          }
        },
        "unit": "events",
        "value": {
          "reason": "not measured in fictional cleanup",
          "state": "not_collected"
        }
      },
      "errors": {
        "method": "fictional cleanup observation",
        "scope": "process",
        "source": {
          "artifact_id": {
            "state": "not_applicable"
          },
          "mechanism": "fictional manual capture",
          "version": {
            "state": "present",
            "value": "1"
          }
        },
        "unit": "events",
        "value": {
          "reason": "not measured in fictional cleanup",
          "state": "not_collected"
        }
      },
      "flushes": {
        "method": "fictional cleanup observation",
        "scope": "process",
        "source": {
          "artifact_id": {
            "state": "not_applicable"
          },
          "mechanism": "fictional manual capture",
          "version": {
            "state": "present",
            "value": "1"
          }
        },
        "unit": "events",
        "value": {
          "reason": "not measured in fictional cleanup",
          "state": "not_collected"
        }
      },
      "partial_writes": {
        "method": "fictional cleanup observation",
        "scope": "process",
        "source": {
          "artifact_id": {
            "state": "not_applicable"
          },
          "mechanism": "fictional manual capture",
          "version": {
            "state": "present",
            "value": "1"
          }
        },
        "unit": "events",
        "value": {
          "reason": "not measured in fictional cleanup",
          "state": "not_collected"
        }
      },
      "retries": {
        "method": "fictional cleanup observation",
        "scope": "process",
        "source": {
          "artifact_id": {
            "state": "not_applicable"
          },
          "mechanism": "fictional manual capture",
          "version": {
            "state": "present",
            "value": "1"
          }
        },
        "unit": "events",
        "value": {
          "reason": "not measured in fictional cleanup",
          "state": "not_collected"
        }
      },
      "stalls": {
        "method": "fictional cleanup observation",
        "scope": "process",
        "source": {
          "artifact_id": {
            "state": "not_applicable"
          },
          "mechanism": "fictional manual capture",
          "version": {
            "state": "present",
            "value": "1"
          }
        },
        "unit": "events",
        "value": {
          "reason": "not measured in fictional cleanup",
          "state": "not_collected"
        }
      }
    },
    "experiment_ref": "EXP-0001",
    "fault_contract": {
      "state": "not_applicable"
    },
    "hypothesis_refs": [
      "HYP-0001"
    ],
    "interval": {
      "clock_id": "fictional-monotonic",
      "elapsed_ns": "0",
      "end": {
        "state": "present",
        "value": "0"
      },
      "method": "fictional cleanup endpoints",
      "precision_ns": {
        "state": "present",
        "value": "1"
      },
      "source": {
        "artifact_id": {
          "state": "not_applicable"
        },
        "mechanism": "fictional manual capture",
        "version": {
          "state": "present",
          "value": "1"
        }
      },
      "start": {
        "state": "present",
        "value": "0"
      },
      "time_domain": "monotonic"
    },
    "io": [
      {
        "bytes": {
          "method": "fictional cleanup observation",
          "scope": "application",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "unit": "bytes",
          "value": {
            "reason": "not measured in fictional cleanup",
            "state": "not_collected"
          }
        },
        "count": {
          "method": "fictional cleanup observation",
          "scope": "application",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "unit": "operations",
          "value": {
            "reason": "not measured in fictional cleanup",
            "state": "not_collected"
          }
        },
        "layer": "application",
        "operation": "read"
      },
      {
        "bytes": {
          "method": "fictional cleanup observation",
          "scope": "application",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "unit": "bytes",
          "value": {
            "reason": "not measured in fictional cleanup",
            "state": "not_collected"
          }
        },
        "count": {
          "method": "fictional cleanup observation",
          "scope": "application",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "unit": "operations",
          "value": {
            "reason": "not measured in fictional cleanup",
            "state": "not_collected"
          }
        },
        "layer": "application",
        "operation": "write"
      },
      {
        "bytes": {
          "method": "fictional cleanup observation",
          "scope": "vfs",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "unit": "bytes",
          "value": {
            "reason": "not measured in fictional cleanup",
            "state": "not_collected"
          }
        },
        "count": {
          "method": "fictional cleanup observation",
          "scope": "vfs",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "unit": "operations",
          "value": {
            "reason": "not measured in fictional cleanup",
            "state": "not_collected"
          }
        },
        "layer": "vfs",
        "operation": "read"
      },
      {
        "bytes": {
          "method": "fictional cleanup observation",
          "scope": "vfs",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "unit": "bytes",
          "value": {
            "reason": "not measured in fictional cleanup",
            "state": "not_collected"
          }
        },
        "count": {
          "method": "fictional cleanup observation",
          "scope": "vfs",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "unit": "operations",
          "value": {
            "reason": "not measured in fictional cleanup",
            "state": "not_collected"
          }
        },
        "layer": "vfs",
        "operation": "write"
      },
      {
        "bytes": {
          "method": "fictional cleanup observation",
          "scope": "filesystem",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "unit": "bytes",
          "value": {
            "reason": "not measured in fictional cleanup",
            "state": "not_collected"
          }
        },
        "count": {
          "method": "fictional cleanup observation",
          "scope": "filesystem",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "unit": "operations",
          "value": {
            "reason": "not measured in fictional cleanup",
            "state": "not_collected"
          }
        },
        "layer": "filesystem",
        "operation": "read"
      },
      {
        "bytes": {
          "method": "fictional cleanup observation",
          "scope": "filesystem",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "unit": "bytes",
          "value": {
            "reason": "not measured in fictional cleanup",
            "state": "not_collected"
          }
        },
        "count": {
          "method": "fictional cleanup observation",
          "scope": "filesystem",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "unit": "operations",
          "value": {
            "reason": "not measured in fictional cleanup",
            "state": "not_collected"
          }
        },
        "layer": "filesystem",
        "operation": "write"
      },
      {
        "bytes": {
          "method": "fictional cleanup observation",
          "scope": "block",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "unit": "bytes",
          "value": {
            "reason": "not measured in fictional cleanup",
            "state": "not_collected"
          }
        },
        "count": {
          "method": "fictional cleanup observation",
          "scope": "block",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "unit": "operations",
          "value": {
            "reason": "not measured in fictional cleanup",
            "state": "not_collected"
          }
        },
        "layer": "block",
        "operation": "read"
      },
      {
        "bytes": {
          "method": "fictional cleanup observation",
          "scope": "block",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "unit": "bytes",
          "value": {
            "reason": "not measured in fictional cleanup",
            "state": "not_collected"
          }
        },
        "count": {
          "method": "fictional cleanup observation",
          "scope": "block",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "unit": "operations",
          "value": {
            "reason": "not measured in fictional cleanup",
            "state": "not_collected"
          }
        },
        "layer": "block",
        "operation": "write"
      },
      {
        "bytes": {
          "method": "fictional cleanup observation",
          "scope": "device",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "unit": "bytes",
          "value": {
            "reason": "not measured in fictional cleanup",
            "state": "not_collected"
          }
        },
        "count": {
          "method": "fictional cleanup observation",
          "scope": "device",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "unit": "operations",
          "value": {
            "reason": "not measured in fictional cleanup",
            "state": "not_collected"
          }
        },
        "layer": "device",
        "operation": "read"
      },
      {
        "bytes": {
          "method": "fictional cleanup observation",
          "scope": "device",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "unit": "bytes",
          "value": {
            "reason": "not measured in fictional cleanup",
            "state": "not_collected"
          }
        },
        "count": {
          "method": "fictional cleanup observation",
          "scope": "device",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "unit": "operations",
          "value": {
            "reason": "not measured in fictional cleanup",
            "state": "not_collected"
          }
        },
        "layer": "device",
        "operation": "write"
      },
      {
        "bytes": {
          "method": "fictional cleanup observation",
          "scope": "other",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "unit": "bytes",
          "value": {
            "reason": "not measured in fictional cleanup",
            "state": "not_collected"
          }
        },
        "count": {
          "method": "fictional cleanup observation",
          "scope": "other",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "unit": "operations",
          "value": {
            "reason": "not measured in fictional cleanup",
            "state": "not_collected"
          }
        },
        "layer": "other",
        "operation": "read"
      },
      {
        "bytes": {
          "method": "fictional cleanup observation",
          "scope": "other",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "unit": "bytes",
          "value": {
            "reason": "not measured in fictional cleanup",
            "state": "not_collected"
          }
        },
        "count": {
          "method": "fictional cleanup observation",
          "scope": "other",
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "unit": "operations",
          "value": {
            "reason": "not measured in fictional cleanup",
            "state": "not_collected"
          }
        },
        "layer": "other",
        "operation": "write"
      }
    ],
    "latency": {
      "algorithm": "none for non-performance cleanup",
      "evidence": {
        "reason": "not measured in fictional cleanup",
        "state": "not_collected"
      },
      "interval": {
        "clock_id": "fictional-monotonic",
        "elapsed_ns": "0",
        "end": {
          "state": "present",
          "value": "0"
        },
        "method": "fictional cleanup endpoints",
        "precision_ns": {
          "state": "present",
          "value": "1"
        },
        "source": {
          "artifact_id": {
            "state": "not_applicable"
          },
          "mechanism": "fictional manual capture",
          "version": {
            "state": "present",
            "value": "1"
          }
        },
        "start": {
          "state": "present",
          "value": "0"
        },
        "time_domain": "monotonic"
      },
      "loss": {
        "state": "present",
        "value": {
          "lost": "0",
          "reason": "no samples expected"
        }
      },
      "method": "fictional cleanup observation",
      "population": "0",
      "rounding": "no rounding performed",
      "unit": "nanoseconds"
    },
    "lifecycle_interval": {
      "clock_id": "fictional-monotonic",
      "elapsed_ns": "0",
      "end": {
        "state": "present",
        "value": "0"
      },
      "method": "fictional cleanup endpoints",
      "precision_ns": {
        "state": "present",
        "value": "1"
      },
      "source": {
        "artifact_id": {
          "state": "not_applicable"
        },
        "mechanism": "fictional manual capture",
        "version": {
          "state": "present",
          "value": "1"
        }
      },
      "start": {
        "state": "present",
        "value": "0"
      },
      "time_domain": "monotonic"
    },
    "logical_bytes": [
      {
        "bytes": {
          "reason": "not measured in fictional cleanup",
          "state": "not_collected"
        },
        "domain": "envelope",
        "method": "fictional cleanup observation"
      },
      {
        "bytes": {
          "reason": "not measured in fictional cleanup",
          "state": "not_collected"
        },
        "domain": "key",
        "method": "fictional cleanup observation"
      },
      {
        "bytes": {
          "reason": "not measured in fictional cleanup",
          "state": "not_collected"
        },
        "domain": "payload",
        "method": "fictional cleanup observation"
      },
      {
        "bytes": {
          "reason": "not measured in fictional cleanup",
          "state": "not_collected"
        },
        "domain": "value",
        "method": "fictional cleanup observation"
      }
    ],
    "memory": {
      "cache": {
        "method": "fictional cleanup observation",
        "scope": "process",
        "source": {
          "artifact_id": {
            "state": "not_applicable"
          },
          "mechanism": "fictional manual capture",
          "version": {
            "state": "present",
            "value": "1"
          }
        },
        "unit": "bytes",
        "value": {
          "reason": "not measured in fictional cleanup",
          "state": "not_collected"
        }
      },
      "peak_resident": {
        "method": "fictional cleanup observation",
        "scope": "process",
        "source": {
          "artifact_id": {
            "state": "not_applicable"
          },
          "mechanism": "fictional manual capture",
          "version": {
            "state": "present",
            "value": "1"
          }
        },
        "unit": "bytes",
        "value": {
          "reason": "not measured in fictional cleanup",
          "state": "not_collected"
        }
      },
      "resident": {
        "method": "fictional cleanup observation",
        "scope": "process",
        "source": {
          "artifact_id": {
            "state": "not_applicable"
          },
          "mechanism": "fictional manual capture",
          "version": {
            "state": "present",
            "value": "1"
          }
        },
        "unit": "bytes",
        "value": {
          "reason": "not measured in fictional cleanup",
          "state": "not_collected"
        }
      },
      "virtual": {
        "method": "fictional cleanup observation",
        "scope": "process",
        "source": {
          "artifact_id": {
            "state": "not_applicable"
          },
          "mechanism": "fictional manual capture",
          "version": {
            "state": "present",
            "value": "1"
          }
        },
        "unit": "bytes",
        "value": {
          "reason": "not measured in fictional cleanup",
          "state": "not_collected"
        }
      }
    },
    "operation_counts": {
      "accepted": {
        "method": "fictional cleanup counter",
        "unit": "operations",
        "value": {
          "state": "present",
          "value": "0"
        }
      },
      "acknowledged": {
        "method": "fictional cleanup counter",
        "unit": "operations",
        "value": {
          "state": "present",
          "value": "0"
        }
      },
      "attempted": {
        "method": "fictional cleanup counter",
        "unit": "operations",
        "value": {
          "state": "present",
          "value": "0"
        }
      },
      "committed": {
        "method": "fictional cleanup counter",
        "unit": "operations",
        "value": {
          "state": "present",
          "value": "0"
        }
      },
      "corrupt": {
        "method": "fictional cleanup counter",
        "unit": "operations",
        "value": {
          "state": "present",
          "value": "0"
        }
      },
      "failed": {
        "method": "fictional cleanup counter",
        "unit": "operations",
        "value": {
          "state": "present",
          "value": "0"
        }
      },
      "missing": {
        "method": "fictional cleanup counter",
        "unit": "operations",
        "value": {
          "state": "present",
          "value": "0"
        }
      },
      "provisional": {
        "method": "fictional cleanup counter",
        "unit": "operations",
        "value": {
          "state": "present",
          "value": "0"
        }
      },
      "recovered": {
        "method": "fictional cleanup counter",
        "unit": "operations",
        "value": {
          "state": "present",
          "value": "0"
        }
      },
      "rejected": {
        "method": "fictional cleanup counter",
        "unit": "operations",
        "value": {
          "state": "present",
          "value": "0"
        }
      },
      "uncertain": {
        "method": "fictional cleanup counter",
        "unit": "operations",
        "value": {
          "state": "present",
          "value": "0"
        }
      }
    },
    "operations": [],
    "phase": {
      "name": "cleanup",
      "observation_role": "non_performance"
    },
    "physical_bytes": [
      {
        "bytes": {
          "reason": "not measured in fictional cleanup",
          "state": "not_collected"
        },
        "domain": "checkpoint",
        "method": "fictional cleanup observation"
      },
      {
        "bytes": {
          "reason": "not measured in fictional cleanup",
          "state": "not_collected"
        },
        "domain": "compaction",
        "method": "fictional cleanup observation"
      },
      {
        "bytes": {
          "reason": "not measured in fictional cleanup",
          "state": "not_collected"
        },
        "domain": "database",
        "method": "fictional cleanup observation"
      },
      {
        "bytes": {
          "reason": "not measured in fictional cleanup",
          "state": "not_collected"
        },
        "domain": "manifest",
        "method": "fictional cleanup observation"
      },
      {
        "bytes": {
          "reason": "not measured in fictional cleanup",
          "state": "not_collected"
        },
        "domain": "other",
        "method": "fictional cleanup observation"
      },
      {
        "bytes": {
          "reason": "not measured in fictional cleanup",
          "state": "not_collected"
        },
        "domain": "read",
        "method": "fictional cleanup observation"
      },
      {
        "bytes": {
          "reason": "not measured in fictional cleanup",
          "state": "not_collected"
        },
        "domain": "requested_io",
        "method": "fictional cleanup observation"
      },
      {
        "bytes": {
          "reason": "not measured in fictional cleanup",
          "state": "not_collected"
        },
        "domain": "sst",
        "method": "fictional cleanup observation"
      },
      {
        "bytes": {
          "reason": "not measured in fictional cleanup",
          "state": "not_collected"
        },
        "domain": "synchronized",
        "method": "fictional cleanup observation"
      },
      {
        "bytes": {
          "reason": "not measured in fictional cleanup",
          "state": "not_collected"
        },
        "domain": "temporary",
        "method": "fictional cleanup observation"
      },
      {
        "bytes": {
          "reason": "not measured in fictional cleanup",
          "state": "not_collected"
        },
        "domain": "wal",
        "method": "fictional cleanup observation"
      },
      {
        "bytes": {
          "reason": "not measured in fictional cleanup",
          "state": "not_collected"
        },
        "domain": "written",
        "method": "fictional cleanup observation"
      }
    ],
    "platform_contract_ref": {
      "artifact_id": "00000000-0000-4000-8000-000000000091",
      "byte_length": "0",
      "sha256": "2222222222222222222222222222222222222222222222222222222222222222",
      "uri": "file:/fictional/a91"
    },
    "producer_record": {
      "artifact_id": "00000000-0000-4000-8000-000000000095",
      "byte_length": "0",
      "sha256": "5555555555555555555555555555555555555555555555555555555555555555",
      "uri": "file:/fictional/a95"
    },
    "profile_id": "fictional-cleanup",
    "provenance": {
      "edge_artifact_ref": {
        "artifact_id": "00000000-0000-4000-8000-000000000099",
        "byte_length": "0",
        "sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "uri": "file:/fictional/a99"
      },
      "endpoint_artifact_ids": [
        "00000000-0000-4000-8000-000000000005",
        "00000000-0000-4000-8000-000000000091",
        "00000000-0000-4000-8000-000000000092",
        "00000000-0000-4000-8000-000000000093",
        "00000000-0000-4000-8000-000000000094",
        "00000000-0000-4000-8000-000000000095",
        "00000000-0000-4000-8000-000000000096",
        "00000000-0000-4000-8000-000000000097",
        "00000000-0000-4000-8000-000000000098",
        "00000000-0000-4000-8000-000000000099"
      ]
    },
    "recovery": {
      "state": "not_applicable"
    },
    "repetition_id": "rep-0",
    "repository": {
      "commit": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
      "dirty_state": "clean",
      "patch_artifact_id": {
        "state": "not_applicable"
      }
    },
    "requirement_refs": [
      "REQ-014"
    ],
    "resource_measurements": [],
    "result_classification": {
      "labels": [
        "valid",
        "diagnostic"
      ],
      "reasons": [
        "corrected fictional classification note"
      ]
    },
    "sample_id": "cleanup-sample-0",
    "sample_population": {
      "included": "0",
      "lost": "0",
      "omission_reason": {
        "state": "not_applicable"
      },
      "total": "0"
    },
    "subject_id": "fictional-subject",
    "synchronization": {
      "completed": {
        "method": "fictional cleanup observation",
        "scope": "process",
        "source": {
          "artifact_id": {
            "state": "not_applicable"
          },
          "mechanism": "fictional manual capture",
          "version": {
            "state": "present",
            "value": "1"
          }
        },
        "unit": "operations",
        "value": {
          "reason": "not measured in fictional cleanup",
          "state": "not_collected"
        }
      },
      "failed": {
        "method": "fictional cleanup observation",
        "scope": "process",
        "source": {
          "artifact_id": {
            "state": "not_applicable"
          },
          "mechanism": "fictional manual capture",
          "version": {
            "state": "present",
            "value": "1"
          }
        },
        "unit": "operations",
        "value": {
          "reason": "not measured in fictional cleanup",
          "state": "not_collected"
        }
      },
      "group_scope": {
        "state": "not_applicable"
      },
      "primitive": "none for cleanup",
      "requested": {
        "method": "fictional cleanup observation",
        "scope": "process",
        "source": {
          "artifact_id": {
            "state": "not_applicable"
          },
          "mechanism": "fictional manual capture",
          "version": {
            "state": "present",
            "value": "1"
          }
        },
        "unit": "operations",
        "value": {
          "reason": "not measured in fictional cleanup",
          "state": "not_collected"
        }
      },
      "wait": {
        "method": "fictional cleanup observation",
        "scope": "process",
        "source": {
          "artifact_id": {
            "state": "not_applicable"
          },
          "mechanism": "fictional manual capture",
          "version": {
            "state": "present",
            "value": "1"
          }
        },
        "unit": "nanoseconds",
        "value": {
          "reason": "not measured in fictional cleanup",
          "state": "not_collected"
        }
      }
    },
    "throughput": {
      "bytes": {
        "denominator_ns": "0",
        "interval": {
          "clock_id": "fictional-monotonic",
          "elapsed_ns": "0",
          "end": {
            "state": "present",
            "value": "0"
          },
          "method": "fictional cleanup endpoints",
          "precision_ns": {
            "state": "present",
            "value": "1"
          },
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "start": {
            "state": "present",
            "value": "0"
          },
          "time_domain": "monotonic"
        },
        "method": "fictional non-performance rate",
        "numerator": "0",
        "unit": "bytes_per_second",
        "value": {
          "reason": "rate not applicable to cleanup",
          "state": "not_collected"
        }
      },
      "operations": {
        "denominator_ns": "0",
        "interval": {
          "clock_id": "fictional-monotonic",
          "elapsed_ns": "0",
          "end": {
            "state": "present",
            "value": "0"
          },
          "method": "fictional cleanup endpoints",
          "precision_ns": {
            "state": "present",
            "value": "1"
          },
          "source": {
            "artifact_id": {
              "state": "not_applicable"
            },
            "mechanism": "fictional manual capture",
            "version": {
              "state": "present",
              "value": "1"
            }
          },
          "start": {
            "state": "present",
            "value": "0"
          },
          "time_domain": "monotonic"
        },
        "method": "fictional non-performance rate",
        "numerator": "0",
        "unit": "operations_per_second",
        "value": {
          "reason": "rate not applicable to cleanup",
          "state": "not_collected"
        }
      }
    },
    "time_meanings": {
      "durability": {
        "reason": "time meaning not applicable to cleanup",
        "state": "not_collected"
      },
      "effective": {
        "reason": "time meaning not applicable to cleanup",
        "state": "not_collected"
      },
      "observation": {
        "reason": "time meaning not applicable to cleanup",
        "state": "not_collected"
      },
      "system_acceptance": {
        "reason": "time meaning not applicable to cleanup",
        "state": "not_collected"
      }
    },
    "validation": {
      "findings": [
        "fictional example is not repository evidence"
      ],
      "integrity": [],
      "status": "not_validated",
      "validated_at_utc_ns": {
        "reason": "fictional example not validated",
        "state": "not_collected"
      },
      "validator_configuration_ref": {
        "reason": "fictional example not validated",
        "state": "not_collected"
      },
      "validator_identity": {
        "reason": "fictional example not validated",
        "state": "not_collected"
      },
      "validator_version": {
        "reason": "fictional example not validated",
        "state": "not_collected"
      }
    },
    "visibility": {
      "first_visible_monotonic_ns": {
        "state": "not_applicable"
      },
      "probe": "not applicable during cleanup",
      "status": "not_observed"
    },
    "workload_ref": {
      "artifact_id": "00000000-0000-4000-8000-000000000092",
      "byte_length": "0",
      "sha256": "3333333333333333333333333333333333333333333333333333333333333333",
      "uri": "file:/fictional/a92"
    }
  },
  "created_at_utc_ns": "2",
  "record_id": "00000000-0000-4000-8000-000000000005",
  "record_kind": "raw_result",
  "run_id": {
    "state": "present",
    "value": "00000000-0000-4000-8000-000000000004"
  },
  "schema_version": "EXP1-R7-JSON-JCS-1",
  "series_id": "00000000-0000-4000-8000-000000000002",
  "supersedes_record_id": {
    "state": "present",
    "value": "00000000-0000-4000-8000-000000000003"
  },
  "correction_reason": {
    "state": "present",
    "value": "corrected fictional classification note"
  }
}
```

## 4. Exact JCS record-domain vector

The exact stored JCS bytes are the following single line, with **no trailing LF**:

```text
{"body":{"byte_length":"0","errors":[],"outcome":"valid","profile_checks":[{"check_id":"empty-artifact-digest","evidence_artifact_id":{"state":"not_applicable"},"message":{"state":"not_applicable"},"outcome":"pass"}],"sha256":"0000000000000000000000000000000000000000000000000000000000000000","validated_artifact_id":"00000000-0000-4000-8000-000000000006","validated_record_id":{"state":"not_applicable"},"validation_started_at_utc_ns":"0","validator_identity":"fictional-independent-validator","validator_version":"1"},"correction_reason":{"state":"not_applicable"},"created_at_utc_ns":"0","record_id":"00000000-0000-4000-8000-000000000007","record_kind":"validation_report","run_id":{"state":"not_applicable"},"schema_version":"EXP1-R7-JSON-JCS-1","series_id":"00000000-0000-4000-8000-000000000002","supersedes_record_id":{"state":"not_applicable"}}
```

Recompute SHA-256 over the 32-byte ASCII/NUL domain prefix `rusty-data-os/exp1/r7/record/v1\0` followed immediately by the 851 record bytes above. The total digest input is 883 bytes. Expected lowercase digest:

```text
86a118d01d7c5b4c21be15cfa60232de6aef06c6bc6a0e6e4d4912ada59557d0
```

Independent audit commands may extract the line without its Markdown newline or reconstruct the displayed JSON object, apply JCS, prefix the domain bytes, and use any conforming SHA-256 implementation. This vector is stable documentation, not a validation report produced by software in this repository.

## 5. Focused invalid cases

Each mutation is applied independently to the exact vector bytes; all other bytes remain unchanged.

| Mutation | Required failure | Why |
|---|---|---|
| Append byte `0a` | `noncanonical` | Stored records admit no trailing newline. |
| Repeat `"record_id"` before the existing member | `duplicate-member` | Duplicate names are rejected before semantic validation. |
| Change `"created_at_utc_ns":"0"` to JSON number `9223372036854775807` | `range` | It is outside the JCS safe-integer number range and this field requires an i64 decimal string. |
| Change it to string `"9223372036854775808"` | `range` | It exceeds signed 64-bit maximum. |
| Change `byte_length` to `"18446744073709551616"` | `range` | It exceeds unsigned 64-bit maximum. |
| Remove `validated_record_id` | `missing-field` | Required missing states are explicit, not absent. |
| Add `"notes2":"x"` to `body` | `unknown-field` | Every object is closed. |
| Swap the two lexical member positions `record_id` and `record_kind` | `noncanonical` | JCS member order is mandatory. |
| Set `validated_record_id` to `{"reason":"x","state":"not_applicable"}` | `type` | `not_applicable` permits no `reason`. |
| Set `outcome` to `"valid"` and add one error | `duplicate-or-conflict` | A valid report requires no errors and all checks passing. |

A future R9 implementation must reproduce these decisions, but this file neither supplies executable fixtures nor authorizes that implementation.

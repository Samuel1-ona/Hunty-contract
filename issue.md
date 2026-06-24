#93 end_time not validated to be in the future at hunt creation
Repo Avatar
Samuel1-ona/Hunty-contract
Validation: create_hunt stores any end_time value without checking end_time > current_time. A creator can accidentally set an already-expired end time, making the hunt immediately inaccessible after activation.
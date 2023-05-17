module codecat_addr::codecat {
	use aptos_framework::account;
	use std::signer;
	use aptos_framework::event;
	use std::string::String;
	use aptos_std::table::{Self, Table};
	use std::option::Option;

	#[test_only]
	use std::string;
	#[test_only]
	use std::option;

	  // Errors
  const E_NOT_INITIALIZED: u64 = 1;
  const ECODE_DOESNT_EXIST: u64 = 2;

	struct CodeList has key {
		codes: Table<String, Code>,
		set_code_event: event::EventHandle<Code>,
		code_counter: u64
	}

	struct Code has store, drop, copy {
    uri: String,
    address:address,
    desc: Option<String>,
  }

	public entry fun register(account: &signer) {
		let signer_address = signer::address_of(account);
		if (!exists<CodeList>(signer_address)){
			let code_list = CodeList {
				codes: table::new(),
				set_code_event: account::new_event_handle<Code>(account),
				code_counter: 0
      };

		  move_to(account, code_list);
		}
	}

  public entry fun add_code(account: &signer, name: String, uri: String, desc: Option<String>) acquires CodeList {
		let signer_address = signer::address_of(account);
		assert!(exists<CodeList>(signer_address), E_NOT_INITIALIZED);
		let code_list = borrow_global_mut<CodeList>(signer_address);
		let code = Code {
			uri,
			address: signer_address,
			desc,
		};
		table::upsert(&mut code_list.codes, name, code);
		code_list.code_counter = code_list.code_counter + 1;
		event::emit_event<Code>(
      &mut borrow_global_mut<CodeList>(signer_address).set_code_event,
      code,
    );
	}

	public entry fun remove_code(account: &signer, name: String) acquires CodeList {
		let signer_address = signer::address_of(account);
		assert!(exists<CodeList>(signer_address), E_NOT_INITIALIZED);
		let code_list = borrow_global_mut<CodeList>(signer_address);

		assert!(table::contains(&code_list.codes, name), ECODE_DOESNT_EXIST);
		let code = table::remove(&mut code_list.codes, name);
		code_list.code_counter = code_list.code_counter - 1;
		event::emit_event<Code>(
      &mut borrow_global_mut<CodeList>(signer_address).set_code_event,
      code,
    );
	}

	#[test(admin = @0x123456)]
  public entry fun test_flow(admin: signer) acquires CodeList {
    account::create_account_for_test(signer::address_of(&admin));
    register(&admin);

    add_code(&admin, string::utf8(b"github"), string::utf8(b"https://github.com/jjwygjj/aptos-codecat.git"), option::some(string::utf8(b"github test")));
    let code_count = event::counter(&borrow_global<CodeList>(signer::address_of(&admin)).set_code_event);
    assert!(code_count == 1, 3);
    let code_list = borrow_global<CodeList>(signer::address_of(&admin));
    assert!(code_list.code_counter == 1, 4);
    let code_record = table::borrow(&code_list.codes, string::utf8(b"github"));
    assert!(code_record.uri == string::utf8(b"https://github.com/jjwygjj/aptos-codecat.git"), 5);
    assert!(code_record.desc == option::some(string::utf8(b"github test")), 6);
    assert!(code_record.address == signer::address_of(&admin), 7);

    remove_code(&admin, string::utf8(b"github"));
    let code_list = borrow_global<CodeList>(signer::address_of(&admin));
    assert!(!table::contains(&code_list.codes, string::utf8(b"github")), 8);
  }
}
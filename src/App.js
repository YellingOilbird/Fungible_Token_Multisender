import 'regenerator-runtime/runtime'
import React, {useRef} from 'react'
import {login, logout} from './utils'
import './global.css'
import './app.css'
import * as nearAPI from 'near-api-js'
import {BN} from 'bn.js'
import Big from 'big.js';
import ReactTooltip from 'react-tooltip';
import ReactFileReader from 'react-file-reader';
import {useDetectOutsideClick} from "./useDetectOutsideClick";
import {PublicKey, serialize} from 'near-api-js/lib/utils'
import {KeyType} from 'near-api-js/lib/utils/key_pair'

import getConfig from './config'
import getAppSettings from './app-settings'

const config = getConfig(process.env.NODE_ENV || 'development');
const appSettings = getAppSettings();

const FRAC_DIGITS = 11;
const gas = 300000000000000; 
const multisender_contract = config.contractName;
//LNC
const decimals = 18;
const one_ft_in_decimals = new BN(Math.pow(10, decimals).toString());

//LNC
function ConvertToYocto(amount) {
    return new BN(amount).mul(one_ft_in_decimals).toString();
}

export default function App() {
    // when the user has not yet interacted with the form, disable the button
    const [sendButtonDisabled, setSendButtonDisabled] = React.useState(true);
    const [sendButtonUnsafeDisabled, setSendButtonUnsafeDisabled] = React.useState(true);
    const [checkButtonVisibility, setCheckButtonVisibility] = React.useState(false);
    const [depositButtonDisabled, setDepositButtonDisabled] = React.useState(true);
    const [withdrawButtonDisabled, setWithdrawButtonDisabled] = React.useState(true);
    const [signOutButtonDisabled, setSignOutButtonDisabled] = React.useState(true);
    const [checkStorageButtonDisabled, setCheckStorageDisabled] = React.useState(true);

    const [textareaPlaceHolderVisibility, setTextareaPlaceHolderVisibility] = React.useState(true);

    const [chunkSize, setChunkSize] = React.useState(14); // or 7

    const navDropdownRef = React.useRef(null);
    const [isNavDropdownActive, setIsNaVDropdownActive] = useDetectOutsideClick(navDropdownRef, false);

    // after submitting the form, we want to show Notification
    const [showNotification, setShowNotification] = React.useState("");

    const [accounts, setAccounts] = React.useState({});
    const [accountsTextArea, setAccountsTextArea] = React.useState("");
    const [deposit, setDeposit] = React.useState(0.0);
    const [total, setTotal] = React.useState(0);
    const [deposit_value, setDepositValue] = React.useState(100);
    const [amount, setAmount] = React.useState(0.0);
    const [chunkProcessingIndex, setChunkProcessingIndex] = React.useState(0);
    const [verified, setVerified] = React.useState(false);

    const handleChange = (event) => {
        setDepositValue(event.target.value);
        console.log('deposit_value:', event.target.value);
    };

    const setButtonsVisibility = (accounts, total, deposit, checkOtherButtons) => {
        if (checkOtherButtons === undefined)
            checkOtherButtons = false;

        const signedIn = window.walletConnection.isSignedIn();
        const accountsLength = accounts ? Object.keys(accounts).length : 0;
        setSignOutButtonDisabled(!signedIn);
        setCheckStorageDisabled(!signedIn || !accountsLength || !verified);
        setDepositButtonDisabled(!signedIn || !accountsLength || !total || deposit-total>=0);
        setWithdrawButtonDisabled(!signedIn || !accountsLength || deposit==0 || !deposit);
        setSendButtonDisabled(!signedIn || !accountsLength || deposit-total<0 || total==0);
        setSendButtonUnsafeDisabled(!signedIn || !accountsLength || deposit-total<0 || total==0);
        setCheckButtonVisibility(!signedIn || !accountsLength);
        const allButtonsDisabled = checkOtherButtons && depositButtonDisabled && sendButtonDisabled;
    };

    const getAccountsText = (accounts) => {
        return Object.keys(accounts).length ?
            Object.keys(accounts).reduce(function (acc, cur) {
                return acc + cur + " " + accounts[cur] + "\r";
            }, "")
            : "";
    };

    const ParsedAccountsList = () => {
        let total = 0;
        let counter = 1;
        return Object.keys(accounts).length ?
            <ul className="accounts">
                {Object.keys(accounts).map(function (account_id) {
                    total += Number(accounts[account_id]);
                    const chuckIndex = Math.floor((counter) / chunkSize);
                    let liClassName = (chuckIndex < chunkProcessingIndex) ? "processed" : "";
                    return <li key={account_id} className={liClassName} data-chunk-index={chuckIndex}>
                        <div className="account" title={account_id}>{counter++}. {AccountTrim(account_id)}</div>
                        <div className="amount">{accounts[account_id]} LNC</div>
                    </li>
                })}
                <TotalValue total={total}/>
            </ul> : null;
    };

    const Header = () => {
        return <div className="nav-container">
            <div className="nav-header">
                <NearLogo/>
                <div className="nav-item user-name">{window.accountId}</div>
                <Deposit/>
                <div className="nav align-right">
                    <NavMenu/>
                    <div className="account-sign-out">
                        <button 
                            disabled={signOutButtonDisabled}
                            className={`verify-button send-button ${signOutButtonDisabled ? "hidden" : ""}`}
                            style={{float: 'right'}} onClick={logout}>
                                Sign out
                        </button>
                    </div>
                </div>
            </div>
        </div>
    };

    const Footer = () => {
        return <div className="footer">
            <div className="github">
                <div className="build-on-near"><a href="https://learnnear.club/lnc-barrel/">BUILD IN LNC BARREL</a></div>
                <div className="brand">LNC {appSettings.appNme} | <a href={appSettings.github}
                                                                      rel="nofollow"
                                                                      target="_blank">Open Source</a></div>
            </div>
            <div className="promo">
                <a>Made by</a> <a href="https://github.com/YellingOilbird" rel="nofollow" target="_blank">GUACHARO</a>
            </div>
        </div>
    };


    const Deposit = () => {
        return deposit && Number(deposit) ?
            <div className="nav user-balance" data-tip="Your internal balance in Multisender App">
                {" App Balance:     " + deposit + " LNC"}
            </div>
            :
            null;
    };

    const NearLogo = () => {
        return <div className="logo-container content-desktop">
            <div className="lnc">

            </div>
            <div className="app-name">
                 {appSettings.appNme}
            </div>
        </div>;
    };

    const NavMenu = () => {
        const onClick = () => setIsNaVDropdownActive(!isNavDropdownActive);

        return (
            <div className="nav-menu container">
                <div className="menu-container">
                    <button onClick={onClick} className="menu-trigger">
                        <span className="network-title">{config.networkId}</span>
                        <div className="network-icon"></div>
                    </button>
                    <nav
                        ref={navDropdownRef}
                        className={`menu ${isNavDropdownActive ? "active" : "inactive"}`}
                    >
                        <ul>
                            <li>
                                <a href={appSettings.urlMainnet}>Mainnet</a>
                            </li>
                            <li>
                                <a href={appSettings.urlTestnet}>Testnet</a>
                            </li>
                        </ul>
                    </nav>
                </div>
            </div>
        );
    };

    const TotalValue = (props) => {
        if (props && props.total)
            return <li key="total" className="total">
                <div className="account">Total</div>
                <div className="amount">{props.total.toFixed(2)} LNC</div>
            </li>;
        else
            return null
    };

    

    let parseAmounts = function (input, pasteInProgress) {
        if (pasteInProgress === undefined)
            pasteInProgress = false;
        /*
        first character: [0-9a-zA-Z]
        account_id: [\_\-0-9a-zA-Z.]*
        separator: [\t,|\||=| ]
        amount ([0-9\.\,]+)
        */
        const pattern = RegExp(/^([0-9a-zA-Z][\_\-0-9a-zA-Z.]*)[\t,|\||=| ]([0-9\.\,]+$)/, 'gm');
        let accounts = {};
        let result;
        let total = 0;
        while ((result = pattern.exec(input)) !== null) {
            const account_name = result[1].toLowerCase();
            const amount = parseFloat(result[2].replace(',', '.').replace(' ', ''))
            if (result[1] && amount) {
                if (accounts.hasOwnProperty(account_name)) {
                    accounts[account_name] += amount;
                } else
                    accounts[account_name] = amount;

                total += amount;
            }
        }
        setTextareaPlaceHolderVisibility(!input.length);
        setTotal(total);
        setAccounts(accounts);
        if (!pasteInProgress) {
            setAccountsTextArea(input);
        }
        if (verified == true) {
            setButtonsVisibility(accounts, total, deposit, true);
        } 
    };

    const ActionButtons = useRef(null)

    const scrollToBottom = () => {
        ActionButtons.current.scrollIntoView({behavior: "smooth"});
    }
    
    const GetDeposit = async () => {

        const deposit = await window.contract.get_deposit({
            account_id: window.accountId
        });

        const depositFormatted = nearAPI.utils.format.formatNearAmount(deposit, FRAC_DIGITS).replace(",", "");
        const depositFormattedInDecimals = (Number(depositFormatted)*1000000).toString();
        setDeposit(depositFormattedInDecimals);
        console.log(depositFormattedInDecimals);
        if (depositFormattedInDecimals == 0) {
            setWithdrawButtonDisabled(true);
        } else {
            setWithdrawButtonDisabled(false);
        }
        return depositFormattedInDecimals;
    };

    // The useEffect hook can be used to fire side-effects during render
    // Learn more: https://reactjs.org/docs/hooks-intro.html
    React.useEffect(
        async () => {
            // in this case, we only care to query the contract when signed in
            if (window.walletConnection.isSignedIn()) {

                await GetDeposit().then((deposit) => {
                    const accountsRaw = JSON.parse(window.localStorage.getItem('accounts'));

                    let accounts = {};
                    if (accountsRaw && accountsRaw.length) {
                        let total = 0;
                        Object.keys(accountsRaw).map(function (index) {
                            const amountFormatted = nearAPI.utils.format.formatNearAmount(accountsRaw[index].amount, FRAC_DIGITS).replace(",", "");
                            const amount = (Number(amountFormatted)*1000000).toString();
                            total += Number(amount);
                            accounts[accountsRaw[index].account_id] = amount;
                        });
                        setTextareaPlaceHolderVisibility(false);
                        setAccounts(accounts);
                        setAccountsTextArea(getAccountsText(accounts));
                        if (verified == true) {
                            setTotal(total);
                            setButtonsVisibility(accounts, total, deposit, true);
                        }
                    }
                });
            }
        },

        // The second argument to useEffect tells React when to re-run the effect
        // Use an empty array to specify "only run on first render"
        // This works because signing into NEAR Wallet reloads the page
        []
    )

    // if not signed in, return early with sign-in prompt
    if (!window.walletConnection.isSignedIn()) {
        return (
            <>
                <Header/>
                <main>
                    <h2 style={{backgroundColor: '--secondary-bg'}}>{appSettings.appNme}</h2>
                    <p>
                        {appSettings.appDescription}
                    </p>
                    <p>
                        To make use of the NEAR blockchain, you need to sign in. The button
                        below will sign you in using NEAR Wallet.
                    </p>
                    <p style={{textAlign: 'center', marginTop: '2.5em'}}>
                        <button onClick={login}>Sign in</button>
                    </p>
                </main>
                <Footer/>
            </>
        )
    }

    const handlePaste = (event) => {
        let {value, selectionStart, selectionEnd} = event.target;
        let pastedValue = event.clipboardData.getData("text");
        let pre = value.substring(0, selectionStart);
        let post = value.substring(selectionEnd, value.length);
        value = (pre + pastedValue + post).trim();
        parseAmounts(value, true);
    };

    return (
        // use React Fragment, <>, to avoid wrapping elements in unnecessary divs
        <>
            <Header/>
            <main>
            <div className="background-img"/>
                <h2>
                    Token Multisender Tool
                </h2>
                <div className="row">
                    <div className="btn upload-csv-button">
                    <button
                    style={{ 
                        borderRadius: '10', 
                        color: '#04AA6D', /* Green background */
                        border: '3px solid gray', /* Green border */
                        color: 'white', /* White text */
                        float: 'left'
                    }}
                    onClick={ async event => {
                    event.preventDefault()
                    ReactTooltip.hide();
                    await window.contractFT.storage_deposit({account_id : window.accountId}, gas, '1250000000000000000000');
                    console.log("success deposited storage...",deposit);
                    }}
                    data-tip={`storage deposit to token contract (like registration)`}
                  >
                  Request LNC Storage
                  </button>
                </div>
                <div className="column">

                    <input
                    style={{ 
                        borderRadius: '10', 
                        color: '#ffb25a', /* Green background */
                        border: '3px solid gray', /* Green border */
                        color: 'black', /* White text */
                        float: 'right',
                        width: '150px',
                        height: '50px',
                        fontSize: '0.8em'
                    }}
                      type = "text"
                      id = "deposit_value"
                      name = "deposit_value"
                      onChange={handleChange}
                      value={deposit_value}
                    />
                <div className='row'>
                <button
                style={{ 
                    borderRadius: '10', 
                    color: '#04AA6D', /* Green background */
                    border: '3px solid gray', /* Green border */
                    color: 'white', /* White text */
                    float: 'right',
                    width: '150px',
                    height: '50px',
                    fontSize: '0.8em'
                }}
                    onClick={ async event => {
                       event.preventDefault()
                        ReactTooltip.hide();
                        const amount = ConvertToYocto(deposit_value);
                        setAmount(amount);
                        await window.contractFT.ft_transfer_call({
                            receiver_id : multisender_contract,
                            amount: amount,
                            msg: ""
                        },
                        gas, Big('0.000000000000000000000001').times(10 ** 24).toFixed());
                    }}
                    data-tip={`Deposit ${deposit_value} tokens to the Multisender App`}
                    >
                    Deposit {deposit_value} LNC
                </button>
                </div>
                    </div>
                </div>
                <br></br>
                <div>
                        <label
                            style={{
                                display: 'block',
                                color: 'var(--gray)',
                            }}
                        >
                            Recipients and amounts
                        </label>
                        <label
                            style={{
                                display: 'block',
                                color: 'var(--gray)',
                                fontSize: '0.6em',
                                marginBottom: '0.5em'
                            }}
                        >
                            Enter one address and amount in token on each line.
                        </label>
                </div>

                <form>
                    <fieldset id="fieldset">
                        <div className="accounts-textarea">
                                  <textarea
                                      autoFocus
                                      autoComplete="off"
                                      id="accounts"
                                      defaultValue={accountsTextArea}
                                      onChange={e => parseAmounts(e.target.value)}
                                      onPaste={e => handlePaste(e)}
                                  />
                            {
                                textareaPlaceHolderVisibility &&
                                <div className="accounts-placeholder">
                                    account1.near 3
                                    <br/>
                                    <br/>
                                    <br/> 
                                    * only supports non-float amounts. floats will rounded automatically!
                                </div>
                            }
                        </div>

                        <div className="action-buttons">
                            <button
                                disabled={checkStorageButtonDisabled}
                                className={`send-button ${checkStorageButtonDisabled ? "hidden" : ""}`}
                                onClick={async event => {
                                    event.preventDefault();
                                    ReactTooltip.hide();

                                    // disable the form while the value gets updated on-chain
                                    fieldset.disabled = true

                                    let multisenderAccounts = Object.keys(accounts).reduce(function (acc, cur) {
                                        acc.push({account_id: cur, amount: ConvertToYocto(accounts[cur])})
                                        return acc;
                                    }, []);

                                    SaveAccountsToLocalStorage(multisenderAccounts);

                                    const allAccountKeys = Object.keys(accounts);
                                    let nonFundedAccounts = [];
                                    let total_already_registered = 0;

                                    const groupSize = 500;
                                    let groupIndex = -1;
                                    let accountGroups = [];
                                    for (let i = 0; i < allAccountKeys.length; i++) {
                                        if (i % groupSize === 0) {
                                            groupIndex++;
                                            accountGroups[groupIndex] = [];
                                        }

                                        accountGroups[groupIndex].push(allAccountKeys[i])
                                    }

                                    let group = 0;
                                    while (group < accountGroups.length) {
                                        let checkAccountGroup = async () => {
                                            return await Promise.all(accountGroups[group].map(async account => {
                                                    let registered = await window.contractFT.storage_balance_of({account_id: account})
                                                    .then();
                                                    if (registered) {
                                                        total_already_registered += 1;
                                                    } else {
                                                        console.log("Not registered account: " + account);
                                                        nonFundedAccounts.push(account);
                                                    }
                                                }
                                            ));
                                        }

                                        await checkAccountGroup().then((validAccounts) => {
                                            Object.values(validAccounts).map(account => {
                                                if (account) {
                                                    nonFundedAccounts[account] = accounts[account];
                                                }
                                            });
                                        });

                                        group++;
                                        console.log("total_registered:" + total_already_registered);
                                    }

                                    const funded = Object.keys(nonFundedAccounts).length;
                                    const total_storage_bond = new BN(funded).mul(new BN("1250000000000000000000")).toString();
                                    console.log("NEED_TO_FUND: "+funded);
                                    console.log("TOTAL_VERIFIED: "+total);
                                    console.log("TOTAL_STORAGE_BOND: "+total_storage_bond);
                                    console.log(nonFundedAccounts);

                                  if (funded > 0) {
                                        await new Promise ((res, _rej) => {
                                            window.contract.multi_storage_deposit({
                                                accounts: nonFundedAccounts
                                            }, 
                                            gas, total_storage_bond);
                                            setTimeout(res, 1000);
                                            setVerified(true);
                                            setShowNotification({
                                                method: "text",
                                                data: `Registered ${funded} account(s)`
                                            });
                                        });
                                  };
                                  delay(1000).then(() => {    
                                    setAccounts(accounts);
                                    setAccountsTextArea(getAccountsText(accounts));
                                    setTotal(total);
                                    console.log("FUNDED: "+funded);
                                    console.log("TOTAL_VERIFIED: "+total);
                                    console.log("TOTAL_STORAGE_BOND: "+total_storage_bond);
                                    console.log(nonFundedAccounts);
                                    setVerified(true);
                                    //setDepositButtonDisabled(false);
                                    setButtonsVisibility(accounts, total, deposit);
                                    GetDeposit();

                                    fieldset.disabled = false
                                    // show Notification
                                    setShowNotification({
                                        method: "text",
                                        data: `All accounts are registered`
                                    });
                                    if (total)
                                        scrollToBottom();
                                    // remove Notification again after css animation completes
                                    // this allows it to be shown again next time the form is submitted
                                    setTimeout(() => {
                                        setShowNotification("")
                                    }, 11000);
                                  })
                                }} 
                            data-tip={"Fund storage for non-registered accounts LIMIT 50 ACCOUNTS"}>
                            Check storage balances *
                            </button>
                            <button
                                disabled={checkButtonVisibility}
                                className={`verify-button send-button ${checkButtonVisibility ? "hidden" : ""}`}
                                onClick={async event => {
                                    event.preventDefault();
                                    ReactTooltip.hide();

                                    // disable the form while the value gets updated on-chain
                                    fieldset.disabled = true

                                    const connection = getNearAccountConnection();
                                    const allAccountKeys = Object.keys(accounts);
                                    let validAccountsFiltered = [];
                                    let total = 0;

                                    const groupSize = 500;
                                    let groupIndex = -1;
                                    let accountGroups = [];
                                    for (let i = 0; i < allAccountKeys.length; i++) {
                                        if (i % groupSize === 0) {
                                            groupIndex++;
                                            accountGroups[groupIndex] = [];
                                        }

                                        accountGroups[groupIndex].push(allAccountKeys[i])
                                    }

                                    let group = 0;
                                    while (group < accountGroups.length) {
                                        let checkAccountGroup = async () => {
                                            return await Promise.all(accountGroups[group].map(async account => {
                                                    let valid = await accountExists(connection, account).then();
                                                    if (valid) {
                                                        return account;
                                                    } else {
                                                        console.log("Invalid account: " + account);
                                                    }
                                                }
                                            ));
                                        }

                                        await checkAccountGroup().then((validAccounts) => {
                                            Object.values(validAccounts).map(account => {
                                                if (account) {
                                                    validAccountsFiltered[account] = accounts[account];
                                                    total += parseFloat(accounts[account]);
                                                }
                                            });
                                        });

                                        group++;
                                    }

                                    const removed = Object.keys(accounts).length - Object.keys(validAccountsFiltered).length;
                                    setAccounts(validAccountsFiltered);
                                    setAccountsTextArea(getAccountsText(validAccountsFiltered));
                                    setTotal(total);
                                    setButtonsVisibility(validAccountsFiltered, 0, deposit);

                                    fieldset.disabled = false
                                    // show Notification
                                    if (removed > 0)
                                        setShowNotification({
                                            method: "text",
                                            data: `Removed ${removed} invalid account(s)`
                                        });
                                    else
                                        setShowNotification({
                                            method: "text",
                                            data: `All accounts are valid`
                                        });

                                    // remove Notification again after css animation completes
                                    // this allows it to be shown again next time the form is submitted
                                    setTimeout(() => {
                                        setShowNotification("")
                                    }, 11000);
                                    setCheckStorageDisabled(false);
                                }}
                                data-tip={"Remove invalid accounts from the list"}>
                                Verify accounts
                            </button>
                        </div>

                        <ParsedAccountsList/>

                        {!sendButtonDisabled && <>
                            <div className="warning-text">Please double check account list and total amount before to
                                send
                                funds.
                            </div>
                            <div className="warning-text">Blockchain transactions are invertible.</div>
                        </>}

                        <div className="action-buttons action-buttons-last" ref={ActionButtons}>
                        <button
                                disabled={sendButtonDisabled}
                                className={`send-button ${sendButtonDisabled ? "hidden" : ""}`}
                                onClick={async event => {
                                    event.preventDefault()
                                    ReactTooltip.hide();

                                    let _chunkSize = 7;
                                    setChunkSize(_chunkSize);
                                    console.log("Chunk size: " + _chunkSize);

                                    // disable the form while the value gets updated on-chain
                                    fieldset.disabled = true

                                    try {
                                        let multisenderAccounts = Object.keys(accounts).reduce(function (acc, cur) {
                                            acc.push({account_id: cur, amount: ConvertToYocto(accounts[cur])})
                                            return acc;
                                        }, []);

                                        SaveAccountsToLocalStorage(multisenderAccounts);

                                        let promises = [];

                                        const chunks = multisenderAccounts.reduce(function (result, _value, index, array) {
                                            if (index % _chunkSize === 0) {
                                                const max_slice = Math.min(index + _chunkSize, multisenderAccounts.length);
                                                result.push(array.slice(index, max_slice));
                                            }
                                            return result;
                                        }, []);

                                        const ret = await (chunks).reduce(
                                            async (promise, chunk, index) => {
                                                return promise.then(async last => {
                                                    const ret = last + 100;
                                                    const max_slice = Math.min((index + 1) * _chunkSize, multisenderAccounts.length);
                                                    const remainingAccounts = multisenderAccounts.slice(max_slice);

                                                    SaveAccountsToLocalStorage(remainingAccounts);

                                                    await new Promise(async (res, _rej) => {
                                                        await window.contract.multisend_from_balance({
                                                            accounts: chunk
                                                        }, gas).then(() => {
                                                            setChunkProcessingIndex(index + 1);
                                                        })

                                                        return setTimeout(res, 100);
                                                    });
                                                    return ret;
                                                })
                                            }, Promise.resolve(0)).then(() => {
                                            setButtonsVisibility([], 0, deposit, true);
                                            setShowNotification({
                                                method: "complete",
                                                data: "multisend_from_balance"
                                            });
                                            GetDeposit();
                                        });
                                    } catch (e) {
                                        alert(
                                            'Something went wrong! \n' +
                                            'Check your browser console for more info.\n' +
                                            e.toString()
                                        )
                                        throw e
                                    } finally {
                                        // re-enable the form, whether the call succeeded or failed
                                        fieldset.disabled = false
                                    }
                                }}
                                data-tip={"Multi send to all recipients using your internal balance of Multusender App  by 7 txs. Your deposit: " + deposit + "LNC"}>
                                Send from App Balance
                            </button>
                            <button
                                disabled={sendButtonUnsafeDisabled}
                                className={`send-button ${sendButtonUnsafeDisabled ? "hidden" : ""}`}
                                onClick={async event => {
                                    event.preventDefault()
                                    ReactTooltip.hide();
                                    //SET CHUNKS
                                    let _chunkSize = 25;
                                    setChunkSize(_chunkSize);
                                    console.log("Chunk size: " + _chunkSize);

                                    // disable the form while the value gets updated on-chain
                                    fieldset.disabled = true

                                    try {
                                        let multisenderAccounts = Object.keys(accounts).reduce(function (acc, cur) {
                                            acc.push({account_id: cur, amount: ConvertToYocto(accounts[cur])})
                                            return acc;
                                        }, []);

                                        SaveAccountsToLocalStorage(multisenderAccounts);

                                        let promises = [];

                                        const chunks = multisenderAccounts.reduce(function (result, _value, index, array) {
                                            if (index % _chunkSize === 0) {
                                                const max_slice = Math.min(index + _chunkSize, multisenderAccounts.length);
                                                result.push(array.slice(index, max_slice));
                                            }
                                            return result;
                                        }, []);

                                        const ret = await (chunks).reduce(
                                            async (promise, chunk, index) => {
                                                return promise.then(async last => {
                                                    const ret = last + 100;
                                                    const max_slice = Math.min((index + 1) * _chunkSize, multisenderAccounts.length);
                                                    const remainingAccounts = multisenderAccounts.slice(max_slice);

                                                    SaveAccountsToLocalStorage(remainingAccounts);

                                                    await new Promise(async (res, _rej) => {
                                                        await window.contract.multisend_from_balance_unsafe({
                                                            accounts: chunk
                                                        }, gas, Big('0.000000000000000000000001').times(10 ** 24).toFixed() ).then(() => {
                                                            setChunkProcessingIndex(index + 1);
                                                        })

                                                        return setTimeout(res, 100);
                                                    });
                                                    return ret;
                                                })
                                            }, Promise.resolve(0)).then(() => {
                                            setButtonsVisibility([], 0, deposit, true);
                                            setShowNotification({
                                                method: "complete",
                                                data: "multisend_from_balance_unsafe"
                                            });
                                            GetDeposit();
                                        });
                                    } catch (e) {
                                        alert(
                                            'Something went wrong! \n' +
                                            'Check your browser console for more info.\n' +
                                            e.toString()
                                        )
                                        throw e
                                    } finally {
                                        // re-enable the form, whether the call succeeded or failed
                                        fieldset.disabled = false
                                    }
                                }}
                                data-tip={"Multi send to all recipients using your internal balance by 25 txs. BETTER GAS EFFICIENCY BY IGNORING TRANSFER STATUS. Always Verify Accounts before."}>
                                Send Unsafe from App Balance / 
                                max_Chunk_size = 25 
                            </button>

                            <button
                                disabled = {depositButtonDisabled}
                                className = {`deposit-button ${depositButtonDisabled ? "hidden" : ""}`}
                                onClick={ async event => {
                                    event.preventDefault()
                                    ReactTooltip.hide();

                                    fieldset.disabled = true;

                                    try {
                                        let multisenderAccounts = Object.keys(accounts).reduce(function (acc, cur) {
                                            acc.push({account_id: cur, amount: ConvertToYocto(accounts[cur])})
                                            return acc;
                                        }, []);

                                        SaveAccountsToLocalStorage(multisenderAccounts);

                                        const amount = ConvertToYocto((total - deposit));
                                        setAmount(amount);
                                        await window.contractFT.ft_transfer_call({
                                            receiver_id : multisender_contract,
                                            amount: amount,
                                            msg: ""
                                        },
                                        gas, Big('0.000000000000000000000001').times(10 ** 24).toFixed());
                                    } catch (e) {
                                        alert(
                                            'Something went wrong! \n' +
                                            'Check your browser console for more info.\n' +
                                            e.toString()
                                        )
                                        throw e
                                    } finally {
                                        // re-enable the form, whether the call succeeded or failed
                                        fieldset.disabled = false
                                    }
                                }}
                                data-tip={`Deposit ${total - deposit} tokens to the Multisender App`}
                            >
                                Deposit {total - deposit} LNC
                            </button>

                            <button
                            disabled = {withdrawButtonDisabled}
                            className = {`deposit-button ${withdrawButtonDisabled ? "hidden" : ""}`}
                                onClick={ async event => {
                                    event.preventDefault()
                                    ReactTooltip.hide();
                                    await window.contract.withdraw_all(
                                        {account_id : window.accountId}, 
                                        gas, Big('0.000000000000000000000001').times(10 ** 24).toFixed());
                                    console.log("success withdraw...",deposit);
                                }}
                                data-tip={`Withdraw all tokens from the Multisender App deposit`}
                            >
                                Withdraw all
                            </button>
                            <br></br>
                            <br></br>
                            <br></br>
                        </div>

                    </fieldset>
                </form>
            </main>

            <Footer/>

            {showNotification && Object.keys(showNotification) &&
            <Notification method={showNotification.method} data={showNotification.data}/>}
            <ReactTooltip/>
        </>
    )
}

function getNearAccountConnection() {
    if (!window.connection) {
        const provider = new nearAPI.providers.JsonRpcProvider(config.nodeUrl);
        window.connection = new nearAPI.Connection(config.nodeUrl, provider, {});
    }
    return window.connection;
}

async function accountExists(connection, accountId) {
    if (accountId.length === 44) {
        let key = new PublicKey({keyType: KeyType.ED25519, data: Buffer.from(accountId, 'hex')});
        return !!(key.toString())
    }

    try {
        await new nearAPI.Account(connection, accountId).state();
        return true;
    } catch (error) {
        return false;
    }
}

function SaveAccountsToLocalStorage(accounts) {
    window.localStorage.setItem('accounts', accounts ? JSON.stringify(accounts) : "[]");
}

// this component gets rendered by App after the form is submitted
function Notification(props) {
    const urlPrefix = `https://explorer.${config.networkId}.near.org/accounts`
    if (props.method === "call")
        return (
            <aside>
                <a target="_blank" rel="noreferrer" href={`${urlPrefix}/${window.accountId}`}>
                    {window.accountId}
                </a>
                {' '/* React trims whitespace around tags; insert literal space character when needed */}
                called method: '{props.data}' in contract:
                {' '}
                <a target="_blank" rel="noreferrer" href={`${urlPrefix}/${window.contract.contractId}`}>
                    {window.contract.contractId}
                </a>
                <footer>
                    <div>✔ Succeeded</div>
                    <div>Just now</div>
                </footer>
            </aside>
        )
    else if (props.method === "complete")
        return (
            <aside>
                Request: '{props.data}' complete! Please check the contract:
                {' '}
                <a target="_blank" rel="noreferrer" href={`${urlPrefix}/${window.contract.contractId}`}>
                    {window.contract.contractId}
                </a>
                <footer>
                    <div>✔ Succeeded</div>
                    <div>Just now</div>
                </footer>
            </aside>
        )
    else if (props.method === "text")
        return (
            <aside>
                {props.data}
                <footer>
                    <div>✔ Succeeded</div>
                    <div>Just now</div>
                </footer>
            </aside>
        )
    else return (
            <aside/>
        )
}

function AccountTrim(account_id) {
    if (account_id.length > 14 + 14 + 1)
        return account_id.slice(0, 14) + '…' + account_id.slice(-14);
    else
        return account_id;
}

function delay(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
}